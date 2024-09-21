use std::sync::Arc;

use axum::{middleware, routing::patch, Router};
use user::{change_email, change_password};

use crate::{middlewares::jwt_authorization, AppState};

mod user;

pub fn routes(shared_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/user/change-password", patch(change_password))
        .route("/user/change-email", patch(change_email))
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            jwt_authorization,
        ))
}
