use super::templating::Templater;
use super::{display, healthcheck, record, tagmgr};
use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use log::debug;

use warp::Filter;

pub fn gen_filters(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    templater: Templater,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Beginning filter intialization");
    gen_display(pool.clone(), templater.clone())
        .or(gen_record_tagged(pool.clone()))
        .or(gen_healthcheck(pool.clone(), templater.clone()))
        .or(gen_get_tags(pool.clone(), templater.clone()))
        .or(gen_post_tag(pool.clone()))
        .or(gen_display_by_tag(pool, templater))
}

// GET /display/
fn gen_display(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    templater: Templater,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Initializing display filter");
    warp::path!("display")
        .and(warp::get())
        .and(with_db(pool))
        .and(with_templater(templater))
        .and_then(display::display_last)
}

// GET /display/:string
fn gen_display_by_tag(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    templater: Templater,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Initializing display_by_tag filter");
    warp::path!("display" / String)
        .and(warp::get())
        .and(with_db(pool))
        .and(with_templater(templater))
        .and_then(|display_url, pool, templater| {
            display::display_last_by_tag(pool, templater, display_url)
        })
}

fn gen_get_tags(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    templater: Templater,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Initializing get_tags filter");
    warp::path!("tags")
        .and(warp::get())
        .and(with_db(pool))
        .and(with_templater(templater))
        .and_then(tagmgr::display_tagmgr)
}

fn gen_post_tag(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Initializing post_tag filter");
    warp::path!("tags")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(with_db(pool))
        .and(warp::body::form())
        //.and_then(|pool, body: HashMap<String, String>|tagmgr::new_tag(pool, body))
        .and_then(tagmgr::new_tag)
}

// POST /record/:string
fn gen_record_tagged(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Initializing record filter");
    warp::path!("record" / String)
        .and(warp::post())
        .and(warp::body::bytes())
        .and(warp::header::headers_cloned())
        .and(with_db(pool))
        .and_then(|url_suffix, body, headers, pool| {
            record::record_webhook(pool, body, headers, url_suffix)
        })
}

fn gen_healthcheck(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    templater: Templater,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Initializing healthcheck filter");
    warp::path!("healthcheck")
        .and(warp::get())
        .and(with_db(pool))
        .and(with_templater(templater))
        .and_then(healthcheck::healthcheck)
}

fn with_db(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
) -> impl Filter<
    Extract = (r2d2::Pool<ConnectionManager<PgConnection>>,),
    Error = std::convert::Infallible,
> + Clone
       + 'static {
    warp::any().map(move || pool.clone())
}

fn with_templater(
    templater: Templater,
) -> impl Filter<Extract = (Templater,), Error = std::convert::Infallible> + Clone + 'static {
    warp::any().map(move || templater.clone())
}

// #[cfg(test)]

// #[test]
// fn test_display_matching() {

//     let filter = gen_display(pool, templater);
//     let value = warp::test::request().path("/display").filter();
// }
