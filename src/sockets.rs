use std::sync::Arc;

use axum::http::header::AUTHORIZATION;
use socketioxide::extract::{SocketRef, State};
use sqlx::{Pool, Postgres};

use crate::{
    auth::{jwt::decode_jwt_payload, registration::User},
    AppState,
};

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

    socket.on("message", print_user)
}

pub async fn print_user(socket: SocketRef, State(state): State<Arc<AppState>>) {
    println!("{:?}", socket.get_user(&state.db_pool).await);
}
