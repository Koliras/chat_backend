use axum::Router;
use dotenv::dotenv;
use socketioxide::SocketIo;
use std::{error::Error, sync::Arc};

use chat_backend::{auth, chat, init_db, sockets::on_connect, user, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db_pool = init_db().await;
    let shared_state = Arc::new(AppState { db_pool });

    let (layer, io) = SocketIo::builder()
        .with_state(shared_state.clone())
        .build_layer();

    io.ns("/", on_connect);

    let app = Router::new()
        .nest("/auth", auth::routes(shared_state.clone()))
        .nest("/chat", chat::routes(shared_state.clone()))
        .nest("/user", user::routes(shared_state.clone()))
        .with_state(shared_state)
        .layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
