extern crate openssl;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate futures;
extern crate handlebars;
extern crate metrics_runtime;
extern crate pretty_env_logger;
extern crate r2d2;
extern crate signal_hook;
extern crate warp;

pub mod config;
pub mod display;
pub mod filters;
pub mod healthcheck;
pub mod model;
pub mod record;
pub mod schema;
pub mod templating;

use config::{get_config, AppConfig};
use diesel::{pg::PgConnection, r2d2::ConnectionManager, Connection};
use futures::channel::oneshot;
use log::Level;
use log::{debug, info, warn};
use metrics::{counter, timing};
use metrics_runtime::{exporters::LogExporter, observers::YamlBuilder, Receiver};
use quanta::Clock;
use std::io::{stdout, Error};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::env;

embed_migrations!();

#[tokio::main(core_threads = 5)]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();
    let clock = Clock::new();
    let init_start = clock.start();
    // Setup shutdown signal handling very early
    let term = init_sighandler();
    // Get our config from the environment
    let config = get_config(&mut env::vars());
    debug!("Got config {:?}", config);

    // Next figure out if our DB is ready and if not, run the migrations to get it ready
    debug!("Checking migration status of database");
    let conn = PgConnection::establish(&config.db_url)
        .expect(&format!("Error connecting to {}", &config.db_url));
    embedded_migrations::run_with_output(&conn, &mut stdout()).expect("Migration must succeed!");

    // Setup metrics facade and logexporter
    init_logging(&config);

    let pool_start = clock.start();
    let pool = fill_pool(&config);
    timing!("init.fill_pool", clock.delta(pool_start, clock.end()));
    let templater = templating::construct_template_singleton();
    let (tx, rx) = oneshot::channel();
    let listen_addr = SocketAddr::new(config.listen_addr, config.listen_port);
    let (addr, server) = warp::serve(filters::gen_filters(pool, templater))
        .bind_with_graceful_shutdown(listen_addr, async {
            rx.await.ok();
        });
    info!(
        "Created server on {}, preparing to spawn onto background thread",
        addr
    );
    timing!("init.time_to_serve", clock.delta(init_start, clock.end()));
    tokio::spawn(server);
    info!("Server task spawned, entering runloop waiting for Ctrl-C");
    // Now that everything important is off the main thread we can begin spinning
    // watching for signal delivery via our atomic bool
    while !term.load(Ordering::Relaxed) {
        // ONLY DO PROVABLY SHORT DURATION TIME-BOUNDED WORK INSIDE THIS LOOP
        // ANY TIME SPENT BLOCKED INSIDE THIS LOOP RISKS PREVENTING SIGNAL PROCESSING
    }
    warn!("Caught shutdown signal!");
    info!("Waiting at most 1 second for requests to finish");
    let timeout = tokio::time::delay_for(Duration::from_secs(1));
    let _ = tx.send(());
    timeout.await;
    Ok(())
}

fn fill_pool(config: &AppConfig) -> r2d2::Pool<ConnectionManager<PgConnection>> {
    info!("Using {} for database", config.db_url);
    let manager = ConnectionManager::<PgConnection>::new(config.db_url.clone());
    counter!("init.fill_pool.max_conns", config.max_conns.into());
    info!("Building pool with {} connections", config.max_conns);
    r2d2::Pool::builder()
        .max_size(config.max_conns)
        .build(manager)
        .unwrap()
}


fn init_logging(config: &AppConfig) {
    // Setup metrics facade and logexporter
    let receiver = Receiver::builder()
        .build()
        .expect("Must be able to build metrics receiver");
    let exporter = LogExporter::new(
        receiver.controller(),
        YamlBuilder::new(),
        Level::Info,
        config.stats_interval,
    );
    receiver.install();
    tokio::spawn(async move {
        exporter.async_run().await;
    });
}

fn init_sighandler() -> Arc<AtomicBool> {
    let term = Arc::new(AtomicBool::new(false));
    let _ = signal_hook::flag::register(signal_hook::SIGTERM, Arc::clone(&term));
    let _ = signal_hook::flag::register(signal_hook::SIGINT, Arc::clone(&term));
    term
}
