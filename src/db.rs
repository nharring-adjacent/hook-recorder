use super::config::AppConfig;
use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use diesel_migrations::embed_migrations;
use log::{debug, info};
use metrics::{counter, timing};
use quanta::Clock;


embed_migrations!();


pub struct DbFacade {
    pool: Option<r2d2::Pool<ConnectionManager<PgConnection>>>,
}

impl DbFacade {
    pub fn new(config: AppConfig) -> DbFacade {
        let pool = DbFacade::fill_pool(&config);
        let facade = DbFacade{pool: Some(pool)};
        facade.check_migrations();
        facade
    }

    pub fn get_pool(&self) -> r2d2::Pool<ConnectionManager<PgConnection>> {
        self.pool.as_ref().unwrap().clone()
    }

    pub fn get_conn(&self) {
        self.pool.as_ref().unwrap().get().unwrap();
    }

    fn check_migrations(&self) {
        debug!("Checking migration status of database");
        let conn = self.pool.as_ref().unwrap().get().unwrap();
        embedded_migrations::run(&conn).expect("Migration must succeed!");
    }

    fn fill_pool(config: &AppConfig) -> r2d2::Pool<ConnectionManager<PgConnection>>{
        let clock = Clock::new();
        let pool_start = clock.start();
        info!("Using {} for database", config.db_url);
        let manager = ConnectionManager::<PgConnection>::new(config.db_url.clone());
        counter!("init.fill_pool.max_conns", config.max_conns.into());
        info!("Building pool with {} connections", config.max_conns);
        let pool = r2d2::Pool::builder()
            .max_size(config.max_conns)
            .build(manager);
        timing!("init.fill_pool", clock.delta(pool_start, clock.end()));
        pool.unwrap()
    }
}
