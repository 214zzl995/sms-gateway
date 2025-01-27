use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use base64::prelude::*;

pub async fn basic_auth(
    State((username, password)): State<(String, String)>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    if let Some(auth_str) = auth_header {
        if let Some(credentials) = auth_str.strip_prefix("Basic ") {
            let decoded = BASE64_STANDARD
                .decode(credentials)
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            let credential_str = String::from_utf8(decoded).map_err(|_| StatusCode::BAD_REQUEST)?;

            let parts: Vec<&str> = credential_str.splitn(2, ':').collect();
            if parts.len() == 2 {
                let username_check = parts[0];
                let password_check = parts[1];

                if username.eq(username_check) && password.eq(password_check) {
                    return Ok(next.run(req).await);
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
