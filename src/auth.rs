use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::BcryptResult;
use jwt_simple::{
    claims::Claims,
    prelude::{HS256Key, MACLike},
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::AppState;

#[derive(Serialize)]
pub struct User {
    id: u64,
    username: String,
    password: String,
    email: String,
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
pub struct UserEmailAndName {
    username: String,
    password: String,
}

pub async fn login(State(state): State<Arc<AppState>>, Json(payload): Json<LoginDto>) -> Response {
    let query_result = sqlx::query_as::<_, UserEmailAndName>(
        "SELECT password, username FROM chat.user WHERE email=$1 LIMIT 1",
    )
    .bind(&payload.email)
    .fetch_optional(&state.db_pool)
    .await;

    let user: Option<UserEmailAndName>;

    match query_result {
        Ok(res) => user = res,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    let is_valid_password: BcryptResult<bool>;

    match user {
        Some(user) => is_valid_password = bcrypt::verify(payload.password, &user.password),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "User with such email or password doesn't exist",
            )
                .into_response()
        }
    }

    match is_valid_password {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(valid) if !valid => (
            StatusCode::UNAUTHORIZED,
            "User with such email or password doesn't exist",
        )
            .into_response(),
        Ok(_) => StatusCode::OK.into_response(),
    }
}

#[derive(Serialize, Deserialize)]
struct JwtPayload {
    username: String,
}

fn create_jwt_token(
    username: String,
    duration: jwt_simple::prelude::Duration,
) -> Result<String, jwt_simple::Error> {
    let key = HS256Key::from_bytes(
        (std::env::var("JWT_SECRET").expect("JWT_SECRET have to be defined")).as_bytes(),
    );
    let claims = Claims::with_custom_claims(JwtPayload { username }, duration);

    key.authenticate(claims)
}
