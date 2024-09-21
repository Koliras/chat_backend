use std::sync::Arc;

use axum::{
    extract::State,
    http::{header::SET_COOKIE, StatusCode},
    response::{AppendHeaders, IntoResponse, Response},
    Extension, Json,
};
use bcrypt::BcryptResult;
use serde::{Deserialize, Serialize};

use crate::AppState;

use super::{jwt::create_jwt_token, registration::User};

#[derive(Deserialize)]
pub struct LoginDto {
    email: String,
    password: String,
}

pub async fn login(State(state): State<Arc<AppState>>, Json(payload): Json<LoginDto>) -> Response {
    pub struct UserPayload {
        id: i64,
        username: String,
        password: String,
    }
    let query_result = sqlx::query_as!(
        UserPayload,
        "SELECT id, password, username FROM chat.user WHERE email=$1 LIMIT 1",
        &payload.email,
    )
    .fetch_optional(&state.db_pool)
    .await;

    let is_valid_password: BcryptResult<bool>;
    let user: UserPayload;

    match query_result {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(res) => match res {
            Some(received_user) => {
                is_valid_password = bcrypt::verify(payload.password, &received_user.password);
                user = received_user;
            }
            None => {
                return (
                    StatusCode::UNAUTHORIZED,
                    "User with such email or password doesn't exist",
                )
                    .into_response()
            }
        },
    }

    let is_valid = if let Ok(valid) = is_valid_password {
        valid
    } else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    if !is_valid {
        return (
            StatusCode::UNAUTHORIZED,
            "User with such email or password doesn't exist",
        )
            .into_response();
    }

    let access_token = create_jwt_token(
        user.id,
        user.username.clone(),
        jwt_simple::prelude::Duration::from_mins(10),
    );

    let access_token = if let Ok(token) = access_token {
        token
    } else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let refresh_token = create_jwt_token(
        user.id,
        user.username,
        jwt_simple::prelude::Duration::from_days(3),
    );
    let refresh_token = if let Ok(token) = refresh_token {
        token
    } else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let headers = AppendHeaders([(SET_COOKIE, format!("Chat-Refresh={refresh_token}"))]);

    (StatusCode::OK, headers, access_token).into_response()
}

#[derive(Serialize)]
pub struct NormalizedUser {
    pub id: i64,
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
