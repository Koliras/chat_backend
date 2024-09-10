use axum::{routing::post, Router};
use dotenv::dotenv;
use std::{error::Error, sync::Arc};

use chat_backend::{
    auth::{login, register},
    init_db, AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db_pool = init_db().await;
    let shared_state = Arc::new(AppState { db_pool });

    let app = Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
