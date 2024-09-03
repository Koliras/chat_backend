use dotenv::dotenv;
use sqlx::{Pool, Postgres};

pub mod auth;

pub struct AppState {
    pub db_pool: Pool<Postgres>,
}

pub async fn init_db() -> Pool<Postgres> {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL have to be declared");
    let db_pool = sqlx::postgres::PgPool::connect(&db_url)
        .await
        .expect("Could not connect to database");
    db_pool
}
