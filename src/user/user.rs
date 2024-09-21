use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Deserialize;

use crate::{
    auth::registration::{User, ValidPassword},
    AppState,
};

#[derive(Deserialize)]
pub struct ChangePassword {
    new_password: String,
    old_password: String,
}

pub async fn change_password(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChangePassword>,
) -> Response {
    match payload.new_password.is_valid_password() {
        Ok(_) => {}
        Err(message) => return (StatusCode::FORBIDDEN, message).into_response(),
    }

    let is_same = bcrypt::verify(&payload.old_password, &user.password);

    let is_same = if let Ok(same) = is_same {
        same
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not change password due to internal reasons",
        )
            .into_response();
    };

    if !is_same {
        return (StatusCode::FORBIDDEN, "Password isn't correct.").into_response();
    }

    if &payload.old_password == &payload.new_password {
        return (
            StatusCode::FORBIDDEN,
            "New password cannot be the same as the old one.",
        )
            .into_response();
    }

    let pass_encrypt_res = bcrypt::hash(&payload.new_password.as_bytes(), 10);

    let password = match pass_encrypt_res {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not change password due to internal reasons",
            )
                .into_response();
        }
    };

    let result = sqlx::query("UPDATE chat.user SET password=$1 WHERE id=$2")
        .bind(password)
        .bind(user.id)
        .execute(&state.db_pool)
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not update the password.".to_string(),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct ChangeEmail {
    new_email: String,
}
pub async fn change_email(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChangeEmail>,
) -> Response {
    todo!()
}
