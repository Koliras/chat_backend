use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{
    auth::{jwt::decode_jwt_payload, User},
    AppState,
};

pub async fn jwt_authorization(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth| auth.to_str().ok());

    let auth_header = if let Some(auth) = auth_header {
        auth
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let split_auth_header: Vec<&str> = auth_header.split(" ").collect();
    if split_auth_header.len() != 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let jwt_payload = decode_jwt_payload(split_auth_header[1]);

    let jwt_payload = if let Ok(payload) = jwt_payload {
        payload
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let user_search = sqlx::query_as!(
        User,
        "SELECT * FROM chat.user WHERE id=$1 LIMIT 1",
        jwt_payload.id,
    )
    .fetch_one(&state.db_pool)
    .await;

    match user_search {
        Ok(user) => {
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
