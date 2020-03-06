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

pub fn main() {}
