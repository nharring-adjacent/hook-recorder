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
pub mod db;
pub mod display;
pub mod filters;
pub mod healthcheck;
pub mod model;
pub mod record;
pub mod schema;
pub mod server;
pub mod templating;

use config::AppConfig;
use db::DbFacade;
use templating::Templater;

use log::Level;
use log::{info, warn};
use metrics::timing;
use metrics_runtime::{exporters::LogExporter, observers::YamlBuilder, Receiver};
use quanta::Clock;
use std::env;
use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// embed_migrations!();

#[tokio::main(core_threads = 5)]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();
    let clock = Clock::new();
    let init_start = clock.start();
    // Setup shutdown signal handling very early
    let term = init_sighandler();
    // Get our config from the environment
    let config = AppConfig::new(&mut env::vars());

    let db = DbFacade::new(config.clone());

    // Setup metrics facade and logexporter
    init_logging(&config);
    let templater = Templater::new();
    // let (tx, rx) = oneshot::channel();
    // let listen_addr = SocketAddr::new(config.listen_addr, config.listen_port);
    // let (addr, server) = warp::serve(filters::gen_filters(pool, templater))
    //     .bind_with_graceful_shutdown(listen_addr, async {
    //         rx.await.ok();
    //     });
    // info!(
    //     "Created server on {}, preparing to spawn onto background thread",
    //     addr
    // );
    let tx = server::spawn_server(db, config, templater);
    timing!("init.time_to_serve", clock.delta(init_start, clock.end()));
    // tokio::spawn(server);
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
