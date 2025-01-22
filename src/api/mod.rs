use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use log::debug;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{db::SMS, Devices};

mod auth;

pub async fn run_api(devices: Devices, server_host: &str, server_port: &u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/logout", get(logout))
        .route("/sms", get(get_sms_paginated))
        .route("/sms/send", post(send_sms).with_state(devices.clone()))
        .route("/device", get(get_all_modem).with_state(devices.clone()))
        .layer(axum::middleware::from_fn(auth::basic_auth));

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", server_host, server_port)).await?;
    debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct Pagination {
    page: u32,
    per_page: u32,
}

#[derive(Serialize)]
pub struct PaginatedSmsResponse {
    data: Vec<SMS>,
    total: i64,
    page: u32,
    per_page: u32,
}

pub async fn get_sms_paginated(Query(pagination): Query<Pagination>) -> Response {
    let (sms_list, total) = match SMS::paginate(pagination.page, pagination.per_page).await {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Send failed: {}", e),
            ).into_response()
        }
    };

    Json(PaginatedSmsResponse {
        data: sms_list,
        total,
        page: pagination.page,
        per_page: pagination.per_page,
    }).into_response()
}

pub async fn send_sms(
    State(devices): State<Devices>,
    Json(payload): Json<SmsPayload>,
) -> impl IntoResponse {
    let modem = devices.get(&payload.modem_id);
    match modem {
        Some(m) => match m.send_sms(&payload.number, &payload.message).await {
            Ok(_) => (StatusCode::OK, "SMS sent").into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Send failed: {}", e),
            )
                .into_response(),
        },
        None => (StatusCode::NOT_FOUND, "Modem not found").into_response(),
    }
}

pub async fn get_all_modem(State(devices): State<Devices>) -> Json<Vec<ModemInfo>> {
    let modem_list = devices
        .values()
        .map(|modem| ModemInfo {
            name: modem.name.clone(),
            com_port: modem.com_port.clone(),
            baud_rate: modem.baud_rate,
        })
        .collect();
    Json(modem_list)
}

pub async fn logout() -> impl IntoResponse {
    (
        StatusCode::UNAUTHORIZED,
        [("WWW-Authenticate", "Basic realm=logout")],
        "Logged out successfully",
    )
}

#[derive(serde::Deserialize)]
pub struct SmsPayload {
    modem_id: String,
    number: String,
    message: String,
}

#[derive(Serialize)]
pub struct ModemInfo {
    pub name: String,
    pub com_port: String,
    pub baud_rate: u32,
}
