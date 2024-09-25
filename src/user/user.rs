use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Deserialize;

use crate::{
    auth::registration::{User, Validity},
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
    if let Err(message) = payload.new_password.is_valid_password() {
        return (StatusCode::FORBIDDEN, message).into_response();
    }

    let is_same = bcrypt::verify(&payload.old_password, &user.password);

    match is_same {
        Ok(same) if same => {}
        Ok(_) => {
            return (StatusCode::FORBIDDEN, "Old password isn't correct.").into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not change password due to internal reasons",
            )
                .into_response();
        }
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
            "Could not update the password".to_string(),
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
    if user.email == payload.new_email {
        return (
            StatusCode::BAD_REQUEST,
            "New email cannot be the same as the old one",
        )
            .into_response();
    }

    if !payload.new_email.is_valid_email() {
        return (StatusCode::BAD_REQUEST, "Your new email is invalid").into_response();
    }

    let update_result = sqlx::query!(
        "UPDATE chat.user SET email = $1 WHERE id = $2",
        payload.new_email,
        user.id
    )
    .execute(&state.db_pool)
    .await;

    match update_result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => match err {
            sqlx::Error::Database(e) if e.is_unique_violation() => {
                (StatusCode::BAD_REQUEST, "This email is already used").into_response()
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not change your email due to internal reasons",
            )
                .into_response(),
        },
    }
}

#[derive(Deserialize)]
pub struct ChangeUsername {
    new_username: String,
}

pub async fn change_username(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChangeUsername>,
) -> Response {
    if user.username == payload.new_username {
        return (
            StatusCode::BAD_REQUEST,
            "New username cannot be the same as the old one",
        )
            .into_response();
    }

    if payload.new_username.len() < 3 {
        return (
            StatusCode::BAD_REQUEST,
            "New username has to be at least 3 characters long",
        )
            .into_response();
    }

    let update_result = sqlx::query!(
        "UPDATE chat.user SET username = $1 WHERE id = $2",
        payload.new_username,
        user.id
    )
    .execute(&state.db_pool)
    .await;

    match update_result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => match err {
            sqlx::Error::Database(e) if e.is_unique_violation() => {
                (StatusCode::BAD_REQUEST, "This username is already used").into_response()
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not change your username due to internal reasons",
            )
                .into_response(),
        },
    }
}
