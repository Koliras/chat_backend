use std::sync::Arc;

use authentication::{get_me, login};
use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use registration::{change_password, register};
use serde::Serialize;

use crate::{middlewares::jwt_authorization, AppState};

pub mod authentication;
pub mod registration;

#[derive(Serialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub email: String,
}

pub fn routes(shared_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route(
            "/auth/me",
            get(get_me).layer(middleware::from_fn_with_state(
                shared_state.clone(),
                jwt_authorization,
            )),
        )
        .route(
            "/auth/change-password",
            patch(change_password).layer(middleware::from_fn_with_state(
                shared_state.clone(),
                jwt_authorization,
            )),
        )
}
