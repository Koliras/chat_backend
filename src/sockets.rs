use std::sync::Arc;

use axum::http::header::AUTHORIZATION;
use member::add_member;
use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef, State};
use sqlx::{types::Uuid, Pool, Postgres};

use crate::{
    auth::{jwt::decode_jwt_payload, registration::User},
    AppState,
};

mod member;

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

pub async fn on_connect(socket: SocketRef) {
    println!("socket connected: {}", socket.id);

    socket.on("join", join_chat_room);
    socket.on("add-user", add_member);
}

#[derive(Deserialize, Debug, Serialize)]
pub struct JoinRoom {
    chat_id: Uuid,
}

async fn join_chat_room(
    socket: SocketRef,
    Data(data): Data<JoinRoom>,
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
            socket.join(format!("{}", chat_id.id)).ok();
            socket
                .emit("success", "Successfully joined the chat room")
                .ok();
        }
        Err(_) => {
            socket.emit("error", "Could not join the chat room").ok();
        }
    }
}
