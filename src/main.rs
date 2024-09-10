use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use std::{error::Error, sync::Arc};

use chat_backend::{
    auth::{get_me, login, register},
    init_db,
    middlewares::jwt_authorization,
    AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db_pool = init_db().await;
    let shared_state = Arc::new(AppState { db_pool });

    let app = Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route(
            "/me",
            get(get_me).layer(middleware::from_fn_with_state(
                shared_state.clone(),
                jwt_authorization,
            )),
        )
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
