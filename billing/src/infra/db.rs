use dotenvy::dotenv;
use std::env;
use sqlx::{PgConnection, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

pub async fn pg() -> Pool<Postgres> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url).await.unwrap()
}