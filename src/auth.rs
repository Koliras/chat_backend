use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

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
) -> StatusCode {
    let pass_encrypt_res = bcrypt::hash(payload.password.as_bytes(), 10);
    let password: String;

    match pass_encrypt_res {
        Ok(hash) => password = hash,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    }

    let result =
        sqlx::query("INSERT INTO chat.user (username, email, password) VALUES ($1, $2, $3)")
            .bind(payload.username)
            .bind(payload.email)
            .bind(password)
            .execute(&state.db_pool)
            .await;

    match result {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            if let Some(code) = e.as_database_error().unwrap().code() {
                if code == "23505" {
                    return StatusCode::CONFLICT;
                }
            }
            StatusCode::UNPROCESSABLE_ENTITY
        }
    }
}
