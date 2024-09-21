use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
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

trait ValidPassword {
    fn is_valid_password(&self) -> Result<(), String>;
}

impl ValidPassword for String {
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
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterUser>,
) -> Response {
    match payload.password.is_valid_password() {
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
