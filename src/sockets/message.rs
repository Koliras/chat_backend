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

#[derive(Serialize)]
struct NormalizedMessage {
    id: Uuid,
    content: String,
    user_id: Uuid,
    created_at: Option<NaiveDateTime>,
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

#[derive(Deserialize)]
pub struct UpdateMessageInput {
    new_content: String,
    message_id: Uuid,
}
pub async fn update_message(
    socket: SocketRef,
    Data(data): Data<UpdateMessageInput>,
    State(state): State<Arc<AppState>>,
) {
    if data.new_content.trim().len() == 0 {
        socket
            .emit("error", "New message content cannot be 0 characters long")
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

    struct UpdatedMessage {
        id: Uuid,
        content: String,
        user_id: Uuid,
        created_at: Option<NaiveDateTime>,
        chat_id: Uuid,
    }
    let update_result = sqlx::query_as!(
        UpdatedMessage,
        "UPDATE chat.message SET content = $1 WHERE id = $2 AND user_id = $3 RETURNING id, content, user_id, created_at, chat_id",
        data.new_content,
        data.message_id,
        user.id
    )
    .fetch_one(&state.db_pool)
    .await;

    match update_result {
        Ok(message) => {
            socket
                .within(message.chat_id.to_string())
                .emit(
                    "updated-message",
                    NormalizedMessage {
                        id: message.id,
                        user_id: message.user_id,
                        created_at: message.created_at,
                        content: message.content,
                    },
                )
                .ok();
        }
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                socket.emit("error", "The message doesn't exist or you are trying to update someone else's message").ok();
            }
            _ => {
                socket.emit("error", "Could not update the message").ok();
            }
        },
    }
}

#[derive(Deserialize, Serialize)]
pub struct DeleteMessage {
    message_id: Uuid,
}
pub async fn delete_message(
    socket: SocketRef,
    Data(data): Data<DeleteMessage>,
    State(state): State<Arc<AppState>>,
) {
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

    let deletion_result = sqlx::query!(
        "DELETE FROM chat.message WHERE id = $1 AND user_id = $2 RETURNING chat_id",
        data.message_id,
        user.id
    )
    .fetch_one(&state.db_pool)
    .await;

    match deletion_result {
        Ok(val) => {
            socket
                .within(val.chat_id.to_string())
                .emit("deleted-message", data)
                .ok();
        }
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                socket.emit("error", "Could not find the message to delete or you are trying to delete someone else's message").ok();
            }
            _ => {
                socket.emit("error", "Could not delete the message").ok();
            }
        },
    }
}
