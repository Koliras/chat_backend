use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use bcrypt::BcryptResult;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{jwt::create_jwt_token, AppState};

#[derive(Serialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct RegisterUser {
    username: String,
    password: String,
    email: String,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterUser>,
) -> Response {
    let pass_encrypt_res = bcrypt::hash(payload.password.as_bytes(), 10);
    let password: String;

    match pass_encrypt_res {
        Ok(hash) => password = hash,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    let result =
        sqlx::query("INSERT INTO chat.user (username, email, password) VALUES ($1, $2, $3)")
            .bind(payload.username)
            .bind(payload.email)
            .bind(password)
            .execute(&state.db_pool)
            .await;

    match result {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => {
            if let Some(code) = e.as_database_error().unwrap().code() {
                if code == "23505" {
                    return (
                        StatusCode::CONFLICT,
                        "User with such nickname or email already exists",
                    )
                        .into_response();
                }
            }
            StatusCode::UNPROCESSABLE_ENTITY.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct LoginDto {
    email: String,
    password: String,
}

#[derive(FromRow)]
pub struct UserPayload {
    id: i64,
    username: String,
    password: String,
}

pub async fn login(State(state): State<Arc<AppState>>, Json(payload): Json<LoginDto>) -> Response {
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

    match is_valid_password {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(valid) if !valid => (
            StatusCode::UNAUTHORIZED,
            "User with such email or password doesn't exist",
        )
            .into_response(),
        Ok(_) => {
            let jwt_result = create_jwt_token(
                user.id,
                user.username,
                jwt_simple::prelude::Duration::from_mins(10),
            );

            match jwt_result {
                Ok(token) => return (StatusCode::OK, token).into_response(),
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }
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
