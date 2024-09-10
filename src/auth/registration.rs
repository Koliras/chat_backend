use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::AppState;

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
    match validate_password(&payload.password) {
        Ok(_) => {}
        Err(message) => return (StatusCode::FORBIDDEN, message).into_response(),
    }
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

fn validate_password(pass: &str) -> Result<(), String> {
    if pass.len() < 8 {
        return Err(
            "This password is too short. It has to be at least 8 characters long.".to_string(),
        );
    }
    let mut contains_numbers = false;
    let mut contains_uppercase = false;
    let mut contains_lowercase = false;
    let mut contains_symbol = false;

    for char in pass.chars() {
        if char.is_numeric() {
            contains_numbers = true;
        }

        if char.is_uppercase() {
            contains_uppercase = true;
        }

        if char.is_lowercase() {
            contains_lowercase = true;
        }

        if !char.is_alphabetic() && !char.is_numeric() {
            contains_symbol = true;
        }
    }

    if !contains_numbers {
        return Err("Password has to contain at least one digit.".to_string());
    }
    if !contains_uppercase {
        return Err("Password has to contain at least one uppercase letter.".to_string());
    }
    if !contains_lowercase {
        return Err("Password has to contain at least one lowercase letter.".to_string());
    }
    if !contains_symbol {
        return Err("Password has to contain at least one special symbol.".to_string());
    }

    Ok(())
}
