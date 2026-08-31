use rocket::http::Status;
use rocket::{get, post};
use rocket::serde::json::Json;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::catalog;

pub const USER_NAME: &str = "spotify";
pub const ACCESS_TOKEN: &str = "spotify";

pub fn user_id() -> Uuid {
    catalog::stable_id("user:spotify")
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
    pub server_id: Option<String>,
    pub has_password: bool,
    pub has_configured_password: bool,
    pub has_configured_easy_password: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthResult {
    pub user: UserDto,
    pub access_token: &'static str,
    pub server_id: String,
}

fn dto() -> UserDto {
    UserDto {
        id: user_id(),
        name: USER_NAME.to_string(),
        server_id: Some(user_id().to_string()),
        has_password: false,
        has_configured_password: false,
        has_configured_easy_password: false,
    }
}

#[post("/Users/AuthenticateByName", format = "json", data = "<body>")]
pub fn authenticate(body: Json<Value>) -> Json<AuthResult> {
    let _ = body; // single static user; credentials are accepted as-is
    Json(AuthResult {
        user: dto(),
        access_token: ACCESS_TOKEN,
        server_id: user_id().to_string(),
    })
}

#[get("/Users/Public")]
pub fn public_users() -> Json<Vec<UserDto>> {
    Json(vec![dto()])
}

#[get("/Users/<requested_id>")]
pub fn get_user(requested_id: Uuid) -> Option<Json<UserDto>> {
    (requested_id == user_id()).then(|| Json(dto()))
}

#[get("/Users/Me")]
pub fn get_me() -> Json<UserDto> {
    Json(dto())
}

#[post("/Sessions/Logout")]
pub fn logout() -> Status {
    Status::NoContent
}

/// Probed by clients during server setup.
#[get("/System/Info/Public")]
pub fn public_system_info() -> Json<Value> {
    Json(serde_json::json!({
        "Id": user_id(),
        "ServerName": "spotify-mcp",
        "ProductName": "Jellyfin Server",
        "Version": "10.8.13",
        "OperatingSystem": "Linux",
    }))
}
