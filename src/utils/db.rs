use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::OnceLock;

static DB_POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn init_db() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://yugabyte@10.0.10.85:5433/ftp")
        .await
        .unwrap();

    DB_POOL.get_or_init(|| pool);
}

pub fn get_db<'a>() -> &'a PgPool {
    DB_POOL.get().unwrap()
}
