use super::config::AppConfig;
use super::db::DbFacade;
use super::filters;
use super::templating::Templater;
use futures::channel::oneshot;
use log::{debug, info};
use std::net::SocketAddr;

pub fn spawn_server(
    db: DbFacade,
    config: AppConfig,
    templater: Templater,
) -> futures::channel::oneshot::Sender<()> {
    debug!("Going to spawn server");
    let (tx, rx) = oneshot::channel();
    let listen_addr = SocketAddr::new(config.listen_addr, config.listen_port);
    let (addr, server) = warp::serve(filters::gen_filters(db.get_pool(), templater))
        .bind_with_graceful_shutdown(listen_addr, async {
            rx.await.ok();
        });
    info!(
        "Created server on {}, preparing to spawn onto background thread",
        addr
    );
    tokio::spawn(server);
    tx
}
