use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

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

pub trait Validity {
    fn is_valid_password(&self) -> Result<(), String>;
    fn is_valid_email(&self) -> bool;
}

impl Validity for String {
    fn is_valid_password(&self) -> Result<(), String> {
        if self.len() < 8 {
            return Err(
                "This password is too short. It has to be at least 8 characters long.".to_string(),
            );
        }
        let mut contains_numbers = false;
        let mut contains_uppercase = false;
        let mut contains_lowercase = false;
        let mut contains_symbol = false;

        for char in self.chars() {
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

    fn is_valid_email(&self) -> bool {
        if self.len() < 6 {
            return false;
        }
        for c in self.chars() {
            if !c.is_ascii() {
                return false;
            }
        }

        let split_email: Vec<&str> = self.split("@").collect();
        if split_email.len() != 2 {
            return false;
        }

        let domain = split_email[1];
        if domain.len() < 4 {
            return false;
        }

        let split_domain: Vec<&str> = domain.split(".").collect();
        if split_domain.len() != 2 {
            return false;
        }

        if split_domain[0].len() == 0 || split_domain[1].len() < 2 {
            return false;
        }

        return true;
    }
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterUser>,
) -> Response {
    if payload.username.len() < 3 {
        return (
            StatusCode::BAD_REQUEST,
            "Your email should be at least 3 characters long",
        )
            .into_response();
    }
    if !payload.email.is_valid_email() {
        return (StatusCode::BAD_REQUEST, "Invalid email").into_response();
    }
    match payload.password.is_valid_password() {
        Ok(_) => {}
        Err(message) => return (StatusCode::BAD_REQUEST, message).into_response(),
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
        Err(e) => match e {
            sqlx::Error::Database(err) if err.is_unique_violation() => (
                StatusCode::CONFLICT,
                "User with such nickname or email already exists",
            )
                .into_response(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not register you due to internal reasons",
            )
                .into_response(),
        },
    }
}
