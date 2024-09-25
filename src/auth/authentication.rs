use std::sync::Arc;

use axum::{
    extract::State,
    http::{header::SET_COOKIE, StatusCode},
    response::{AppendHeaders, IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::AppState;

use super::{jwt::create_jwt_token, registration::User};

#[derive(Deserialize)]
pub struct LoginDto {
    email: String,
    password: String,
}

pub async fn login(State(state): State<Arc<AppState>>, Json(payload): Json<LoginDto>) -> Response {
    struct UserPayload {
        id: Uuid,
        username: String,
        password: String,
    }
    let query_result = sqlx::query_as!(
        UserPayload,
        "SELECT id, password, username FROM chat.user WHERE email=$1 LIMIT 1",
        &payload.email,
    )
    .fetch_one(&state.db_pool)
    .await;

    let user = match query_result {
        Ok(res) => res,
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                return (
                    StatusCode::UNAUTHORIZED,
                    "User with such email or password doesn't exist",
                )
                    .into_response()
            }
            _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
    };

    let is_valid_password = bcrypt::verify(payload.password, &user.password);

    match is_valid_password {
        Ok(is_valid) if is_valid => {}
        Ok(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                "User with such email or password doesn't exist",
            )
                .into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    let access_token = create_jwt_token(
        user.id,
        user.username.clone(),
        jwt_simple::prelude::Duration::from_mins(10),
    );

    let access_token = match access_token {
        Ok(token) => token,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let refresh_token = create_jwt_token(
        user.id,
        user.username,
        jwt_simple::prelude::Duration::from_days(3),
    );
    let refresh_token = match refresh_token {
        Ok(token) => token,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let headers = AppendHeaders([(SET_COOKIE, format!("Chat-Refresh={refresh_token}"))]);

    (StatusCode::OK, headers, access_token).into_response()
}

#[derive(Serialize)]
pub struct NormalizedUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

pub async fn get_me(Extension(user): Extension<User>) -> Json<NormalizedUser> {
    Json(NormalizedUser {
        id: user.id,
        username: user.username,
        email: user.email,
    })
}
