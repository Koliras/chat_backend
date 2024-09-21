use std::sync::Arc;

use authentication::{get_me, login};
use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use registration::{change_password, register};

use crate::{middlewares::jwt_authorization, AppState};

pub mod authentication;
pub mod jwt;
pub mod registration;

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
