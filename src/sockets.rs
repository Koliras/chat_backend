use std::sync::Arc;

use axum::http::header::AUTHORIZATION;
use member::{add_member, leave_chat, remove_member};
use message::{delete_message, send_message, update_message};
use serde::{Deserialize, Serialize};
use socketioxide::extract::{SocketRef, State, TryData};
use sqlx::{types::Uuid, Pool, Postgres};

use crate::{
    auth::{jwt::decode_jwt_payload, registration::User},
    AppState,
};

mod member;
mod message;

pub trait GetUser {
    fn get_user(
        &self,
        executor: &Pool<Postgres>,
    ) -> impl std::future::Future<Output = Option<User>>;
}

impl GetUser for SocketRef {
    async fn get_user(&self, executor: &Pool<Postgres>) -> Option<User> {
        let auth_header = self
            .req_parts()
            .headers
            .get(AUTHORIZATION)
            .and_then(|auth| auth.to_str().ok());

        let auth_header = match auth_header {
            Some(auth) => auth,
            None => return None,
        };

        let split_header: Vec<&str> = auth_header.split(" ").collect();
        if split_header.len() != 2 {
            return None;
        }

        let jwt_payload = decode_jwt_payload(split_header[1]);

        let jwt_payload = match jwt_payload {
            Ok(payload) => payload,
            Err(_) => return None,
        };

        let user_search = sqlx::query_as!(
            User,
            "SELECT * FROM chat.user WHERE id = $1 LIMIT 1",
            jwt_payload.id
        )
        .fetch_one(executor)
        .await
        .ok();

        return user_search;
    }
}

mod socket_event {
    pub const JOIN: &'static str = "join";
    pub const ADD_USER: &'static str = "add-user";
    pub const REMOVE_USER: &'static str = "remove-user";
    pub const LEAVE_CHAT: &'static str = "leave-chat";
    pub const SEND_MESSAGE: &'static str = "send-message";
    pub const UPDATE_MESSAGE: &'static str = "update-message";
    pub const DELETE_MESSAGE: &'static str = "delete-message";
}

pub async fn on_connect(socket: SocketRef) {
    socket.on(socket_event::JOIN, join_chat_room);
    socket.on(socket_event::ADD_USER, add_member);
    socket.on(socket_event::REMOVE_USER, remove_member);
    socket.on(socket_event::LEAVE_CHAT, leave_chat);
    socket.on(socket_event::SEND_MESSAGE, send_message);
    socket.on(socket_event::UPDATE_MESSAGE, update_message);
    socket.on(socket_event::DELETE_MESSAGE, delete_message);
}

#[derive(Deserialize, Debug, Serialize)]
pub struct JoinRoom {
    chat_id: Uuid,
}

async fn join_chat_room(
    socket: SocketRef,
    TryData(data): TryData<JoinRoom>,
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
        Some(user) => user,
        None => {
            socket
                .emit("error", "Could not authenticate the user by auth header")
                .ok();
            return;
        }
    };

    struct ChatId {
        id: Uuid,
    }
    let query_result = sqlx::query_as!(
        ChatId,
        "SELECT chat_id as id FROM chat.user_chat WHERE chat_id = $1 AND user_id = $2",
        data.chat_id,
        user.id
    )
    .fetch_one(&state.db_pool)
    .await;

    match query_result {
        Ok(chat_id) => {
            socket.leave_all().ok();
            socket.join(chat_id.id.to_string()).ok();
            socket
                .emit("success", "Successfully joined the chat room")
                .ok();
        }
        Err(_) => {
            socket.emit("error", "Could not join the chat room").ok();
        }
    }
}
