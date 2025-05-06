use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use log::debug;
use log::error;
use mime_guess::from_path;
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{db::{Contact, Conversation, SMS}, modem::SmsType, Devices};

mod auth;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Asset;

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
        .route(
            "/device",
            get(get_all_modem_details).with_state(devices.clone()),
        )
        .route(
            "/device/{name}/sms/count",
            get(get_device_sms_count).with_state(devices.clone()),
        )
        .route(
            "/refresh/{name}",
            get(refresh_sms).with_state(devices.clone()),
        )
        .route(
            "/contacts",
            get(get_contacts),
        )
        .route(
            "/contacts",
            post(create_contact),
        )
        .route(
            "/conversation",
            get(get_conversation),
        )
        .layer(axum::middleware::from_fn_with_state(
            (username.to_string(), password.to_string()),
            auth::basic_auth,
        ));

    let app = Router::new()
        .nest_service("/api", api)
        .fallback(static_handler);

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
    contact_id: Option<i64>,
}

pub async fn get_sms_paginated(Query(query): Query<SmsQuery>) -> Response {
    let result = match &query.contact_id {
        Some(contact_id) => SMS::paginate_by_contact_id(contact_id, query.page, query.per_page).await,
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
        Some(m) => match m.send_sms_pdu(&payload.contact, &payload.message).await {
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

pub async fn get_all_modem_details(State(devices): State<Devices>) -> Response {
    fn to_data_error<T, E: ToString>(result: Result<T, E>) -> (Option<T>, Option<String>) {
        match result {
            Ok(data) => (Some(data), None),
            Err(e) => (None, Some(e.to_string())),
        }
    }

    let mut details = Vec::new();

    for (name, modem) in devices.iter() {
        let (signal_data, signal_error) = to_data_error(modem.get_signal_quality().await);
        let (network_data, network_error) = to_data_error(modem.check_network_registration().await);
        let (operator_data, operator_error) = to_data_error(modem.check_operator().await);
        let (model_data, model_error) = to_data_error(modem.get_modem_model().await);

        details.push(json!({
            "name": name,
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
        }));
    }

    (StatusCode::OK, Json(details)).into_response()
}


pub async fn refresh_sms(
    Path(name): Path<String>,
    State(devices): State<Devices>,
) -> Response {
    match devices.get(&name) {
        Some(modem) => {
            match modem.read_sms_sync_insert(SmsType::RecUnread).await{
                Ok(_) => (StatusCode::OK).into_response(),
                Err(err) => (StatusCode::BAD_GATEWAY,err.to_string()).into_response(),
            }
        }
        None => (StatusCode::NOT_FOUND, "Modem not found").into_response(),
    }
}

pub async fn check() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

pub async fn get_contacts() -> Json<Vec<Contact>> {
    let contacts = Contact::query_all().await.unwrap();
    Json(contacts)
}

pub async fn get_conversation() -> Json<Vec<Conversation>> {
    let conversation = Conversation::query_all().await.unwrap();
    Json(conversation)
}

async fn get_device_sms_count(Path(name): Path<String>) -> Response  {
    match SMS::count_by_device(&name).await {
        Ok(count) => (StatusCode::OK, Json(count)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_contact(Json(payload): Json<String>) -> Response {
    match Contact::insert(&payload).await {
        Ok(id) => (StatusCode::OK, Json(id)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    match Asset::get(path) {
        Some(content) => {
            let mime = from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(content.data.into())
                .unwrap()
        }
        None => {
            if let Some(index) = Asset::get("index.html") {
                Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(index.data.into())
                    .unwrap()
            } else {
                (StatusCode::NOT_FOUND, "File not found").into_response()
            }
        }
    }
}

#[derive(serde::Deserialize)]
pub struct SmsPayload {
    modem_id: String,
    contact: Contact,
    message: String,
}

#[derive(Serialize)]
pub struct ModemInfo {
    pub name: String,
    pub com_port: String,
    pub baud_rate: u32,
}
