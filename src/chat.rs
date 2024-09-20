use std::sync::Arc;

use axum::{middleware, routing::get, Router};
use chat::{create_chat, delete_chat, get_chats};

use crate::middlewares::jwt_authorization;
use crate::AppState;

pub mod chat;

pub fn routes(shared_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/chat",
            get(get_chats).post(create_chat).delete(delete_chat),
        )
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            jwt_authorization,
        ))
}
