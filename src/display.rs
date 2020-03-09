extern crate chrono;
extern crate diesel;
use super::model::*;
use super::schema::tags;
use super::schema::tags::dsl::*;
use super::schema::webhooks;
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
    let result = webhooks
        .select(super::schema::webhooks::all_columns)
        .order_by(id.desc())
        .first::<Webhook>(&pool.get().unwrap())
        .expect("Error fetching most recent webhook");
    timing!(
        "display.display_last.webhooks_query",
        clock.delta(query_start, clock.end())
    );
    debug!("Got webhook: {:?}", &result);
    counter!(
        "display.display_last.webhook_size",
        mem::size_of_val(&result).try_into().unwrap()
    );
    let render_start = clock.start();
    let html = templater.hb.render("display", &result);
    timing!(
        "display.display_last.render_time",
        clock.delta(render_start, clock.end())
    );
    Ok(warp::reply::html(
        html.unwrap_or_else(|err| err.to_string()),
    ))
}

pub async fn display_last_by_tag(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    templater: Templater,
    display_url: String,
) -> Result<impl warp::Reply, Infallible> {
    let tag_id_val: i32 = tags
        .filter(url_suffix.eq(display_url))
        .select(tags::tag_id)
        .first(&pool.get().unwrap())
        .unwrap();
    let webhook_for_tag = webhooks
        .filter(webhooks::tag_id.eq(tag_id_val))
        .order_by(upload_time.desc())
        .first::<Webhook>(&pool.get().unwrap())
        .unwrap();
    let html = templater.hb.render("display", &webhook_for_tag);
    Ok(warp::reply::html(
        html.unwrap_or_else(|err| err.to_string()),
    ))
}
