use super::diesel::prelude::RunQueryDsl;
use super::model::{NewWebhook, Webhook};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
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
    tag: Option<String>,
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
    let type_coercion_end = clock.end();
    timing!(
        "record.record_webhook.type_coercion",
        (clock.delta(type_coercion_start, type_coercion_end) / 1000) / 1000
    );
    let result = _do_record_webhook(&pool.get().unwrap(), &headers, &body, tag);

    if result.upload_time.timestamp() > 0 {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::from_u16(500).unwrap())
    }
}

// The private function where we shove strings into text fields into pgsql
fn _do_record_webhook(
    conn: &PooledConnection<ConnectionManager<PgConnection>>,
    headers: &str,
    body: &str,
    tag_val: Option<String>,
) -> Webhook {
    use super::schema::webhooks;
    let tag = tag_val.as_deref();
    let newdoc = NewWebhook { headers, body, tag };

    diesel::insert_into(webhooks::table)
        .values(&newdoc)
        .get_result::<Webhook>(conn)
        .expect("Error saving new webhook POST")
}
