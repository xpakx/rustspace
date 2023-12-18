use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::info;

pub async fn get_db(db_url: &str) -> PgPool {
    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();
 
    info!("Connection to database established.");
    
    info!("Applying migrations...");
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap();
    pool
}
