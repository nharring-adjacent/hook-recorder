use super::diesel::prelude::RunQueryDsl;
use super::model::{NewWebhook, Tag, Webhook};
use super::schema::tags::dsl::*;
use crate::diesel::ExpressionMethods;
use crate::diesel::QueryDsl;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use log::debug;
use metrics::{counter, timing};
use quanta::Clock;
use std::convert::Infallible;
use std::convert::TryInto;

use std::mem;
use warp::http::HeaderMap;
use warp::http::StatusCode;

// This wrapper handles type conversions from the aggregated buffer and HeaderMap the filters give us
pub async fn record_webhook(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    body_bytes: bytes::Bytes,
    header_map: HeaderMap,
    url_seen: String,
) -> Result<impl warp::Reply, Infallible> {
    let clock = Clock::new();
    let type_coercion_start = clock.start();
    counter!(
        "record.record_webhook.header_map.bytes",
        mem::size_of_val(&header_map).try_into().unwrap()
    );
    counter!(
        "record.record_webhook.body_bytes.bytes",
        mem::size_of_val(&body_bytes).try_into().unwrap()
    );
    let headers = format!("{:?}", header_map); // TODO: Use serde_json to derive a string serializer
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    counter!(
        "record.record_webhook.body_string.bytes",
        mem::size_of_val(&body).try_into().unwrap()
    );
    counter!(
        "record.record_webhook.header_string.bytes",
        mem::size_of_val(&headers).try_into().unwrap()
    );
    timing!(
        "record.record_webhook.type_coercion",
        clock.delta(type_coercion_start, clock.end())
    );
    let tag_match_start = clock.start();
    debug!("Finding tag id for url_suffix: {}", url_seen);
    let found = find_tag_id(&pool, url_seen).await;
    let found_tag_id = match found {
        Ok(found_id) => found_id,
        Err(diesel::result::Error::NotFound) => return Ok(StatusCode::from_u16(404).unwrap()),
        Err(_) => return Ok(StatusCode::from_u16(500).unwrap()),
    };
    timing!(
        "record.record_webhook.find_tag_id",
        clock.delta(tag_match_start, clock.end())
    );
    let db_write_start = clock.start();
    let result = _do_record_webhook(&pool, &headers, &body, found_tag_id).await;
    timing!(
        "record.record_webhook.db_write",
        clock.delta(db_write_start, clock.end())
    );
    // Quick'n'dirty proxy that the row got inserted successfully
    if result.upload_time.timestamp() > 0 {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::from_u16(500).unwrap())
    }
}

// The private function where we shove strings into text fields into pgsql
async fn _do_record_webhook(
    pool: &r2d2::Pool<ConnectionManager<PgConnection>>,
    headers: &str,
    body: &str,
    found_tag_id: i32,
) -> Webhook {
    use super::schema::webhooks;
    let newdoc = NewWebhook {
        headers,
        body,
        tag_id: found_tag_id,
    };
    diesel::insert_into(webhooks::table)
        .values(&newdoc)
        .get_result::<Webhook>(&pool.get().unwrap())
        .expect("Error saving new webhook POST")
}

async fn find_tag_id(
    pool: &r2d2::Pool<ConnectionManager<PgConnection>>,
    url_seen: String,
) -> Result<i32, diesel::result::Error> {
    let tag = tags
        .filter(url_suffix.eq(url_seen))
        .first::<Tag>(&pool.get().unwrap())?;
    if tag.active {
        Ok(tag.tag_id)
    } else {
        Err(diesel::result::Error::NotFound)
    }
}
