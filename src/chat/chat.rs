use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{auth::registration::User, AppState};

#[derive(Deserialize)]
pub struct CreateChat {
    name: String,
}

pub async fn create_chat(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateChat>,
) -> Response {
    struct ChatId {
        id: i64,
    }

    let tx = state.db_pool.begin().await;
    let mut tx = match tx {
        Ok(transaction) => transaction,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Could not create chat").into_response();
        }
    };

    let insertion_result = sqlx::query_as!(
        ChatId,
        "INSERT INTO chat.chat (name, admin_id) VALUES ($1, $2) RETURNING id;",
        payload.name,
        user.id
    )
    .fetch_one(&mut *tx)
    .await;

    let chat_id = if let Ok(id) = insertion_result {
        id
    } else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Could not create chat").into_response();
    };

    let insertion_result =
        sqlx::query("INSERT INTO chat.user_chat (user_id, chat_id) VALUES ($1, $2);")
            .bind(user.id)
            .bind(chat_id.id)
            .execute(&mut *tx)
            .await;

    if let Err(_) = insertion_result {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Could not create chat").into_response();
    }

    let tx_result = tx.commit().await;

    match tx_result {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Could not create chat").into_response(),
    }
}

#[derive(FromRow, Serialize)]
pub struct Chat {
    name: String,
    chat_id: i64,
    admin_username: String,
}

pub async fn get_chats(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let query_result = sqlx::query_as!(
        Chat,
        "
        SELECT c.name AS name, c.id AS chat_id, u.username AS admin_username
        FROM chat.user_chat AS uc
        INNER JOIN chat.chat AS c
        ON uc.chat_id = c.id
        INNER JOIN chat.user AS u
        ON u.id = c.admin_id
        WHERE uc.user_id = $1;
        ",
        user.id
    )
    .fetch_all(&state.db_pool)
    .await;

    match query_result {
        Ok(chats) => (StatusCode::OK, Json(chats)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Could not find any chats").into_response(),
    }
}

#[derive(Deserialize)]
pub struct DeleteChatPayload {
    chat_id: i64,
}

pub async fn delete_chat(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeleteChatPayload>,
) -> Response {
    struct ChatId {
        id: Option<i64>,
    }
    let tx = state.db_pool.begin().await;
    let mut tx = match tx {
        Ok(transaction) => transaction,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Could not delete chat").into_response();
        }
    };

    let deletion_result = sqlx::query_as!(
        ChatId,
        "DELETE FROM chat.chat WHERE id = $1 AND admin_id = $2 RETURNING id;",
        payload.chat_id,
        user.id
    )
    .fetch_one(&mut *tx)
    .await;

    if let Ok(a) = deletion_result {
        if a.id.is_none() {
            return (
                StatusCode::NOT_FOUND,
                "Could not find chat with such an id or you are not an admin of this chat",
            )
                .into_response();
        }
    } else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Could not find chat").into_response();
    };

    let deletion_result = sqlx::query!(
        "DELETE FROM chat.user_chat WHERE chat_id = $1;",
        payload.chat_id
    )
    .execute(&mut *tx)
    .await;

    if let Err(_) = deletion_result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not delete users of chat",
        )
            .into_response();
    }

    let deletion_result = sqlx::query!(
        "DELETE FROM chat.message WHERE chat_id = $1;",
        payload.chat_id
    )
    .execute(&mut *tx)
    .await;

    match deletion_result {
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not delete chat messages",
        )
            .into_response(),
        Ok(_) => {
            let commit_result = tx.commit().await;
            match commit_result {
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Could not delete chat and other related entities",
                )
                    .into_response(),
                Ok(_) => StatusCode::NO_CONTENT.into_response(),
            }
        }
    }
}
