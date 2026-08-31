use rocket::http::Status;
use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::{get, post};
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
    pub session_info: SessionInfo,
    pub access_token: &'static str,
    pub server_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionInfo {
    pub id: &'static str,
    pub play_state: Value,
    pub now_playing_item: Option<Value>,
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
        session_info: SessionInfo {
            id: "spotify-mcp",
            play_state: serde_json::json!({}),
            now_playing_item: None,
        },
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
    system_info()
}

/// Probed by clients after authentication to confirm the API is responsive.
#[get("/System/Info")]
pub fn system_info() -> Json<Value> {
    Json(serde_json::json!({
        "Id": user_id(),
        "ServerName": "spotify-mcp",
        "ProductName": "Jellyfin Server",
        "Version": "10.8.13",
        "OperatingSystem": "Linux",
    }))
}

/// Lightweight liveness endpoint used by some Jellyfin clients.
#[get("/System/Ping")]
pub fn ping() -> Json<Value> {
    Json(serde_json::json!({ "Status": "OK" }))
}

/// Fintunes opens the server URL in a WebView and reads these values from the
/// Jellyfin web client's local storage instead of calling AuthenticateByName.
#[get("/")]
pub fn web_client_bootstrap() -> RawHtml<String> {
    let user_id = user_id();
    let server_id = user_id.to_string();
    let credentials = serde_json::json!({
        "Servers": [{
            "ManualAddress": "__SERVER_ORIGIN__",
            "ManualAddressOnly": true,
            "Id": server_id,
            "UserId": user_id,
            "AccessToken": ACCESS_TOKEN,
            "LocalAddress": "__SERVER_ORIGIN__"
        }]
    });
    let credentials = serde_json::to_string(&credentials).expect("credentials are serializable");
    RawHtml(format!(
        "<!doctype html><meta charset=\"utf-8\"><title>spotify-mcp</title><script>const c={credentials:?}.replaceAll('__SERVER_ORIGIN__',location.origin);localStorage.setItem('jellyfin_credentials',c);localStorage.setItem('_deviceId2','spotify-mcp-fintunes');</script>"
    ))
}
