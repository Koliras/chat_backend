use std::sync::Arc;

use authentication::{get_me, login};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use registration::register;

use crate::{middlewares::jwt_authorization, AppState};

pub mod authentication;
pub mod jwt;
pub mod registration;

pub fn routes(shared_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route(
            "/me",
            get(get_me).layer(middleware::from_fn_with_state(
                shared_state.clone(),
                jwt_authorization,
            )),
        )
}
