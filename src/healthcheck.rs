extern crate chrono;
extern crate diesel;
extern crate handlebars;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use log::debug;
use metrics::{counter, timing};
use quanta::Clock;
use serde::{Deserialize, Serialize};
use std::convert::{Infallible};
use std::time::Duration;
use super::templating::TemplateSingleton;

#[derive(Serialize, Deserialize)]
struct HealthcheckPayload {
    healthy: bool,
    conns: u32,
    idle_conns: u32,
    max_conns: u32,
    idle_timeout: Option<Duration>,
    max_lifetime: Option<Duration>,
    conn_timeout: Duration,
}

pub async fn healthcheck(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>, templater: TemplateSingleton,
) -> Result<impl warp::Reply, Infallible> {
    debug!("Healthcheck called");
    let clock = Clock::new();
    let check_start = clock.start();
    let payload = _do_check_health(pool);
    let check_end = clock.end();
    timing!(
        "healthcheck.state_check.ms",
        (clock.delta(check_start, check_end) / 1000) / 1000
    );
    counter!("healthcheck.db_state.conns", payload.conns.into());
    counter!(
        "healthcheck.db_state.idle_conns",
        payload.idle_conns.into()
    );
    let html = templater.hb.render("healthcheck.html", &payload);
    Ok(warp::reply::html(
        html.unwrap_or_else(|err| err.to_string()),
    ))
}

fn _do_check_health(pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> HealthcheckPayload {
    let state = pool.state();
    let mut healthy = false;
    if state.connections > 0 && state.idle_connections > 0 {
        healthy = true;
    }
    HealthcheckPayload {
        healthy: healthy,
        conns: state.connections,
        idle_conns: state.idle_connections,
        max_conns: pool.max_size(),
        idle_timeout: pool.idle_timeout(),
        max_lifetime: pool.max_lifetime(),
        conn_timeout: pool.connection_timeout(),
    }
}
