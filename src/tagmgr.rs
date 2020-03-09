extern crate chrono;
extern crate diesel;
use super::model::{NewTag, Tag};
use super::schema::tags::dsl::*;
use super::templating::Templater;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use log::debug;
use metrics::{counter, timing};
use quanta::Clock;
use std::collections::HashMap;
use std::convert::Infallible;
use std::convert::TryInto;
use serde::{Deserialize, Serialize};
use warp::http::StatusCode;

#[derive(Serialize, Deserialize)]
struct TagsPayload {
    tag_count: u32,
    tags: Vec<Tag>,
}

pub async fn display_tagmgr(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    templater: Templater,
) -> Result<impl warp::Reply, Infallible> {
    let clock = Clock::new();
    debug!("Beginning display tags request");
    let tags_get_start = clock.start();
    let live_tags = tags
        .filter(active.eq(true))
        .order_by(created_at)
        .limit(50)
        .load::<Tag>(&pool.get().unwrap())
        .unwrap();
    timing!(
        "tagmgr.display_tagmgr.live_tags_query",
        clock.delta(tags_get_start, clock.end())
    );
    debug!("Got {} tags", live_tags.len());
    counter!(
        "tagmgr.display_tagmgr.tag_count",
        live_tags.len().try_into().unwrap()
    );
    let count: u32 = live_tags.len().try_into().unwrap_or(0);
    let payload = TagsPayload {
        tag_count: count,
        tags: live_tags,
    };
    let html = templater.hb.render("tags", &payload);
    Ok(warp::reply::html(
        html.unwrap_or_else(|err| err.to_string()),
    ))
}

pub async fn new_tag(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    body: HashMap<String, String>,
) -> Result<impl warp::Reply, Infallible> {
    let tag_val = match body.get("tag") {
        Some(val) => val,
        None => return Ok(StatusCode::from_u16(500).unwrap()),
    };
    let ret = write_new_tag(pool.clone(), tag_val.to_string()).await;
    match ret {
        Ok(_) => Ok(StatusCode::from_u16(200).unwrap()),
        Err(_) => Ok(StatusCode::from_u16(500).unwrap()),
    }
}

async fn write_new_tag(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    tag: String,
) -> Result<Tag, diesel::result::Error> {
    use super::schema::tags;
    let newtag = NewTag {
        url_suffix: tag,
        active: true,
    };
    let tag = diesel::insert_into(tags::table)
        .values(&newtag)
        .get_result::<Tag>(&pool.get().unwrap())?;
    Ok(tag)
}
