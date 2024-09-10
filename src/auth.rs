use serde::Serialize;

pub mod authentication;
pub mod registration;

#[derive(Serialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub email: String,
}
