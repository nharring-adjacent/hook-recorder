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
use log::{debug, info, warn};
use metrics::timing;
use metrics_runtime::{
    exporters::HttpExporter, exporters::LogExporter, observers::YamlBuilder, Receiver,
};
use quanta::Clock;
use std::env;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[tokio::main]
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

    // The return here is a transmit handle to signal shutdown of the warp server
    let tx = server::spawn_server(db, config, Templater::new());
    timing!("init.time_to_serve", clock.delta(init_start, clock.end()));
    info!("Server task spawned, entering runloop waiting for shutdown signal");
    // Now that everything important is running asynchronously on a threadpool
    // we can spin waiting for shutdown
    while !term.load(Ordering::Relaxed) {
        // DO NOT DO ANY WORK INSIDE THIS LOOP OR YOU RISK INTERFERING WITH SIGNALS
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
    let log_exporter = LogExporter::new(
        receiver.controller(),
        YamlBuilder::new(),
        Level::Info,
        config.stats_interval,
    );
    let http_exporter = HttpExporter::new(
        receiver.controller(),
        YamlBuilder::new(),
        SocketAddr::new(config.listen_addr, config.http_stats_port),
    );
    receiver.install();
    if config.enable_stats_logger {
        debug!("Spawning stats logger onto threadpool");
        tokio::spawn(async move {
            log_exporter.async_run().await;
        });
    } else {
        debug!("Stats logging disabled.");
    }
    debug!("Spawning http stats exporter onto threadpool");
    tokio::spawn(async move {
        http_exporter
            .async_run()
            .await
            .expect("Stat request exploded");
    });
}

fn init_sighandler() -> Arc<AtomicBool> {
    let term = Arc::new(AtomicBool::new(false));
    let _ = signal_hook::flag::register(signal_hook::SIGTERM, Arc::clone(&term));
    let _ = signal_hook::flag::register(signal_hook::SIGINT, Arc::clone(&term));
    term
}
