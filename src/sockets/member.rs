use std::sync::Arc;

use serde::Deserialize;
use socketioxide::extract::{SocketRef, State, TryData};
use uuid::Uuid;

use crate::{sockets::GetUser, AppState};

#[derive(Deserialize)]
pub struct ChatMembershipInput {
    user_id: Uuid,
    chat_id: Uuid,
}

pub async fn add_member(
    socket: SocketRef,
    TryData(data): TryData<ChatMembershipInput>,
    State(state): State<Arc<AppState>>,
) {
    let data = match data {
        Ok(data) => data,
        Err(_) => {
            socket.emit("error", "Could not parse body. Please, make sure you have all the required fields with correct names").ok();
            return;
        }
    };
    let user = match socket.get_user(&state.db_pool).await {
        Some(user) if user.id != data.user_id => user,
        Some(_) => {
            socket
                .emit("error", "You cannot add yourself to the chat")
                .ok();
            return;
        }
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
                        "You cannot add users to chat you yourself are not the part of",
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

pub async fn remove_member(
    socket: SocketRef,
    TryData(data): TryData<ChatMembershipInput>,
    State(state): State<Arc<AppState>>,
) {
    let data = match data {
        Ok(data) => data,
        Err(_) => {
            socket.emit("error", "Could not parse body. Please, make sure you have all the required fields with correct names").ok();
            return;
        }
    };
    let user = socket.get_user(&state.db_pool).await;
    let user = match user {
        Some(user) if user.id != data.user_id => user,
        Some(_) => {
            socket
                .emit("error", "User cannot remove himself from the chat")
                .ok();
            return;
        }
        None => {
            socket
                .emit("error", "Could not authenticate the user by auth header")
                .ok();
            return;
        }
    };

    match user.is_admin(&state.db_pool, data.chat_id).await {
        Ok(is_admin) if is_admin => {}
        Ok(_) => {
            socket
                .emit("error", "Only admin can remove other users from the chat")
                .ok();
            return;
        }
        Err(_) => {
            socket
                .emit(
                    "error",
                    "Could not validate that you are an admin of the chat",
                )
                .ok();
            return;
        }
    }

    let deletion_result = sqlx::query!(
        "DELETE FROM chat.user_chat WHERE user_id = $1 AND chat_id = $2 RETURNING user_id",
        data.user_id,
        data.chat_id
    )
    .fetch_one(&state.db_pool)
    .await;

    match deletion_result {
        Ok(_) => {
            socket
                .emit("success", "Successfully removed user from the chat")
                .ok();
            return;
        }
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                socket.emit("error", "Could not find user in chat").ok();
                return;
            }
            _ => {
                socket
                    .emit("error", "Could not remove user from the chat")
                    .ok();
            }
        },
    }
}

#[derive(Deserialize)]
pub struct LeaveChatInput {
    chat_id: Uuid,
}
pub async fn leave_chat(
    socket: SocketRef,
    TryData(data): TryData<LeaveChatInput>,
    State(state): State<Arc<AppState>>,
) {
    let data = match data {
        Ok(data) => data,
        Err(_) => {
            socket.emit("error", "Could not parse body. Please, make sure you have all the required fields with correct names").ok();
            return;
        }
    };
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

    match user.is_admin(&state.db_pool, data.chat_id).await {
        Ok(is_admin) if !is_admin => {}
        Ok(_) => {
            socket
                .emit("error", "Admin cannot leave their own chat")
                .ok();
            return;
        }
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                socket.emit("error", "Could not find you in this chat").ok();
                return;
            }
            _ => {
                socket
                    .emit("error", "Could not check if you are an admin of the chat")
                    .ok();
                return;
            }
        },
    }

    let deletion_result = sqlx::query!(
        "DELETE FROM chat.user_chat WHERE user_id = $1 AND chat_id = $2 RETURNING user_id",
        user.id,
        data.chat_id
    )
    .fetch_one(&state.db_pool)
    .await;

    match deletion_result {
        Ok(_) => {
            socket.emit("success", "Successfully leaved the chat").ok();
            return;
        }
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                socket.emit("error", "Could not find you in this chat").ok();
                return;
            }
            _ => {
                socket.emit("error", "Could not leave the chat").ok();
            }
        },
    }
}
