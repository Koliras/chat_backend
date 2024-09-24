use std::sync::Arc;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef, State};
use sqlx::types::Uuid;

use crate::{sockets::GetUser, AppState};

#[derive(Deserialize)]
pub struct SendMessageInput {
    content: String,
    chat_id: Uuid,
}
pub async fn send_message(
    socket: SocketRef,
    Data(data): Data<SendMessageInput>,
    State(state): State<Arc<AppState>>,
) {
    if data.content.trim().len() == 0 {
        socket
            .emit("error", "Message cannot be 0 characters long")
            .ok();
        return;
    }
    let user = socket.get_user(&state.db_pool).await;
    let user = match user {
        Some(user) => user,
        None => {
            socket
                .emit("error", "Could not authenticate the user by auth header")
                .ok();
            return;
        }
    };

    let result = sqlx::query!(
        "SELECT EXISTS (SELECT 1 FROM chat.user_chat WHERE user_id = $1 AND chat_id = $2)",
        user.id,
        data.chat_id
    )
    .fetch_one(&state.db_pool)
    .await;

    match result {
        Ok(val) => match val.exists {
            Some(in_chat) if in_chat => {}
            Some(_) => {
                socket
                    .emit(
                        "error",
                        "You send messages to the chat you yourself are not the part of",
                    )
                    .ok();
                return;
            }
            None => {
                socket
                    .emit("error", "Failed to check if you are in the chat")
                    .ok();
                return;
            }
        },
        Err(_) => {
            socket
                .emit("error", "Failed to check if you are in the chat")
                .ok();
            return;
        }
    };

    #[derive(Serialize)]
    struct NormalizedMessage {
        id: Uuid,
        content: String,
        user_id: Uuid,
        created_at: Option<NaiveDateTime>,
    }
    let create_message = sqlx::query_as!(
        NormalizedMessage,
        "INSERT INTO chat.message (content, user_id, chat_id) VALUES ($1, $2, $3) RETURNING id, content, user_id, created_at",
        data.content.trim(),
        user.id,
        data.chat_id
    )
    .fetch_one(&state.db_pool)
    .await;

    match create_message {
        Ok(message) => {
            socket
                .within(data.chat_id.to_string())
                .emit("new-message", message)
                .ok();
        }
        Err(_) => {
            socket.emit("error", "Could not send a message").ok();
        }
    }
}
