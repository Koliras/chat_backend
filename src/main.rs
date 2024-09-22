use axum::Router;
use dotenv::dotenv;
use std::{error::Error, sync::Arc};

use chat_backend::{auth, chat, init_db, user, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db_pool = init_db().await;
    let shared_state = Arc::new(AppState { db_pool });

    let app = Router::new()
        .merge(auth::routes(shared_state.clone()))
        .merge(chat::routes(shared_state.clone()))
        .merge(user::routes(shared_state.clone()))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
