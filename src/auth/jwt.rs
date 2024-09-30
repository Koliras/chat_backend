use jwt_simple::{
    claims::Claims,
    prelude::{HS256Key, MACLike},
};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Serialize, Deserialize)]
pub struct JwtPayload {
    pub id: Uuid,
    pub username: String,
}

pub fn create_jwt_token(
    id: Uuid,
    username: String,
    duration: jwt_simple::prelude::Duration,
) -> Result<String, jwt_simple::Error> {
    let key = HS256Key::from_bytes(
        (std::env::var("JWT_SECRET").expect("JWT_SECRET have to be defined")).as_bytes(),
    );
    let claims = Claims::with_custom_claims(JwtPayload { username, id }, duration);

    key.authenticate(claims)
}

pub fn decode_jwt_payload(jwt_token: &str) -> Result<JwtPayload, jwt_simple::Error> {
    let key = HS256Key::from_bytes(
        (std::env::var("JWT_SECRET").expect("JWT_SECRET have to be defined")).as_bytes(),
    );

    let claims = key.verify_token::<JwtPayload>(jwt_token, None)?;

    Ok(JwtPayload {
        username: claims.custom.username,
        id: claims.custom.id,
    })
}
