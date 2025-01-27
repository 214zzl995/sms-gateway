use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use log::debug;
use log::error;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{db::SMS, Devices};

mod auth;

pub async fn run_api(
    devices: Devices,
    server_host: &str,
    server_port: &u16,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let api = Router::new()
        .route("/check", get(check))
        .route("/sms", get(get_sms_paginated))
        .route("/sms", post(send_sms).with_state(devices.clone()))
        .route("/device", get(get_all_modem).with_state(devices.clone()))
        .route(
            "/device/{name}",
            get(get_modem_detail).with_state(devices.clone()),
        )
        .layer(axum::middleware::from_fn_with_state(
            (username.to_string(), password.to_string()),
            auth::basic_auth,
        ));

    let app = Router::new().nest_service("/api", api);

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", server_host, server_port)).await?;
    debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Serialize)]
pub struct PaginatedSmsResponse {
    data: Vec<SMS>,
    total: i64,
    page: u32,
    per_page: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SmsQuery {
    page: u32,
    per_page: u32,
    #[serde(default)]
    device: Option<String>, // 新增可选设备参数
}

pub async fn get_sms_paginated(Query(query): Query<SmsQuery>) -> Response {
    let result = match &query.device {
        Some(device) => SMS::paginate_by_device(device, query.page, query.per_page).await,
        None => SMS::paginate(query.page, query.per_page).await,
    };

    let (sms_list, total) = match result {
        Ok(res) => res,
        Err(e) => {
            error!("{}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get SMS: {}", e),
            )
                .into_response();
        }
    };

    Json(PaginatedSmsResponse {
        data: sms_list,
        total,
        page: query.page,
        per_page: query.per_page,
    })
    .into_response()
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

pub async fn get_modem_detail(
    Path(name): Path<String>,
    State(devices): State<Devices>,
) -> Response {
    match devices.get(&name) {
        Some(modem) => {
            fn to_data_error<T, E: ToString>(result: Result<T, E>) -> (Option<T>, Option<String>) {
                match result {
                    Ok(data) => (Some(data), None),
                    Err(e) => (None, Some(e.to_string())),
                }
            }
    
            // 处理每个异步结果
            let (signal_data, signal_error) = to_data_error(modem.get_signal_quality().await);
            let (network_data, network_error) = to_data_error(modem.check_network_registration().await);
            let (operator_data, operator_error) = to_data_error(modem.check_operator().await);
            let (model_data, model_error) = to_data_error(modem.get_modem_model().await);
    
            // 构建JSON响应
            let response = json!({
                "signal_quality": {
                    "data": signal_data,
                    "error": signal_error
                },
                "network_registration": {
                    "data": network_data,
                    "error": network_error
                },
                "operator": {
                    "data": operator_data,
                    "error": operator_error
                },
                "modem_model": {
                    "data": model_data,
                    "error": model_error
                }
            });
    
            // 返回200 OK及JSON数据
            (StatusCode::OK, Json(response)).into_response()
        }
        None => {
            // 设备未找到，返回404
            (StatusCode::NOT_FOUND, "Modem not found").into_response()
        }
    }
}

pub async fn check() -> impl IntoResponse {
    StatusCode::NO_CONTENT
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
