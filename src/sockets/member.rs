use std::sync::Arc;

use serde::Deserialize;
use socketioxide::extract::{Data, SocketRef, State};
use uuid::Uuid;

use crate::{sockets::GetUser, AppState};

#[derive(Deserialize)]
pub struct AddToChat {
    user_id: Uuid,
    chat_id: Uuid,
}

pub async fn add_member(
    socket: SocketRef,
    Data(data): Data<AddToChat>,
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

    if data.user_id == user.id {
        socket
            .emit("error", "You cannot add yourself to the chat")
            .ok();
        return;
    }

    let result = sqlx::query!(
        "SELECT EXISTS (SELECT 1 FROM chat.user_chat WHERE user_id = $1 AND chat_id = $2)",
        user.id,
        data.chat_id
    )
    .fetch_one(&state.db_pool)
    .await;

    let in_chat = match result {
        Ok(val) => match val.exists {
            Some(b) => b,
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

    if !in_chat {
        socket
            .emit(
                "error",
                "You cannot add users to chat you yourself are not the part of",
            )
            .ok();
        return;
    }

    let add_user = sqlx::query!(
        "INSERT INTO chat.user_chat (user_id, chat_id) VALUES($1, $2)",
        data.user_id,
        data.chat_id
    )
    .execute(&state.db_pool)
    .await;

    match add_user {
        Ok(_) => {
            socket
                .emit("success", "Successfully added the user to the chat")
                .ok();
        }
        Err(e) => match e {
            sqlx::Error::Database(e) => match e.kind() {
                sqlx::error::ErrorKind::UniqueViolation => {
                    socket.emit("error", "User is already in the chat").ok();
                }
                sqlx::error::ErrorKind::ForeignKeyViolation => {
                    socket
                        .emit("error", "User with such an id does not exist")
                        .ok();
                }
                _ => {
                    socket
                        .emit(
                            "error",
                            "Could not add user to the chat due to internal reasons",
                        )
                        .ok();
                }
            },
            _ => {
                socket
                    .emit(
                        "error",
                        "Could not add user to the chat due to internal reasons",
                    )
                    .ok();
            }
        },
    }
}
