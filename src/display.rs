extern crate chrono;
extern crate diesel;
use super::model::*;
use super::schema::webhooks::dsl::*;
use super::templating::Templater;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use log::debug;
use metrics::{counter, timing};
use quanta::Clock;
use std::convert::Infallible;
use std::convert::TryInto;
use std::mem;

pub async fn display_last(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    templater: Templater,
) -> Result<impl warp::Reply, Infallible> {
    let clock = Clock::new();
    debug!("Beginning display request");
    // Get the most recent upload from the db
    let query_start = clock.start();
    let results = webhooks
        .limit(1)
        .order_by(upload_time.desc())
        .load::<Webhook>(&pool.get().unwrap())
        .expect("Error fetching most recent webhook");
    let query_end = clock.end();
    timing!(
        "display.display_last.webhooks_query",
        (clock.delta(query_start, query_end) / 1000) / 1000
    );
    debug!("Got webhook: {:?}", &results[0]);
    counter!(
        "display.display_last.webhook_size",
        mem::size_of_val(&results[0]).try_into().unwrap()
    );
    let render_start = clock.start();
    let html = templater.hb.render("display.html", &results[0]);
    let render_end = clock.end();
    timing!(
        "display.display_last.render_time",
        (clock.delta(render_start, render_end) / 1000) / 1000
    );
    Ok(warp::reply::html(
        html.unwrap_or_else(|err| err.to_string()),
    ))
}
