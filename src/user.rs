use std::sync::Arc;

use axum::{middleware, routing::patch, Router};
use user::{change_email, change_password, change_username};

use crate::{middlewares::jwt_authorization, AppState};

mod user;

pub fn routes(shared_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/change-password", patch(change_password))
        .route("/change-email", patch(change_email))
        .route("/change-username", patch(change_username))
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            jwt_authorization,
        ))
}
