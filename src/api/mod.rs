use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    extract::{Path, Query, State},
    response::{sse::Event, IntoResponse, Response, Sse},
    routing::{delete, get, post},
    Json, Router,
};
use futures_util::StreamExt;
use log::debug;
use log::error;
use mime_guess::from_path;
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
pub use sse_manager::SseManager;

use crate::{
    db::{Contact, Conversation, SMS},
    modem::SmsType,
    Devices,
};

mod auth;
mod sse_manager;

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
    sse_manager: Arc<SseManager>,
) -> anyhow::Result<()> {
    let api = Router::new()
        .route("/check", get(check))
        .route("/sms", get(get_sms_paginated))
        .route("/sms", post(send_sms).with_state(devices.clone()))
        .route("/sms/sse", get(sse_events).with_state(sse_manager.clone()))
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
        .route("/contacts", get(get_contacts))
        .route("/contacts", post(create_contact))
        .route("/contacts/{id}", delete(delete_contact_by_id))
        .route("/conversation", get(get_conversation))
        .route("/conversations/{id}/unread", post(get_conversation_unread))
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
    contact_id: Option<String>,
}

async fn get_sms_paginated(Query(query): Query<SmsQuery>) -> Response {
    let result = match &query.contact_id {
        Some(contact_id) => {
            SMS::paginate_by_contact_id(contact_id, query.page, query.per_page).await
        }
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

async fn send_sms(
    State(devices): State<Devices>,
    Json(mut payload): Json<SmsPayload>,
) -> impl IntoResponse {
    let modem = devices.get(&payload.modem_id);    if payload.new {
        payload.contact.find_or_create().await.unwrap();
    }

    match modem {
        Some(m) => match m.send_sms_pdu(&payload.contact, &payload.message).await {
            Ok((sms_id, contact_id)) => (
                StatusCode::OK,
                Json(json!({ "sms_id": sms_id, "contact_id": contact_id })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Send failed: {}", e),
            )
                .into_response(),
        },
        None => (StatusCode::NOT_FOUND, "Modem not found").into_response(),
    }
}

async fn get_all_modem_details(State(devices): State<Devices>) -> Response {
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

async fn refresh_sms(Path(name): Path<String>, State(devices): State<Devices>) -> Response {
    match devices.get(&name) {
        Some(modem) => match modem.read_sms_sync_insert(SmsType::RecUnread).await {
            Ok(_) => (StatusCode::OK).into_response(),
            Err(err) => (StatusCode::BAD_GATEWAY, err.to_string()).into_response(),
        },
        None => (StatusCode::NOT_FOUND, "Modem not found").into_response(),
    }
}

async fn check() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

async fn get_contacts() -> Json<Vec<Contact>> {
    let contacts = Contact::query_all().await.unwrap();
    Json(contacts)
}

async fn get_conversation() -> Json<Vec<Conversation>> {
    let conversation = Conversation::query_all().await.unwrap();
    Json(conversation)
}

async fn get_device_sms_count(Path(name): Path<String>) -> Response {
    match SMS::count_by_device(&name).await {
        Ok(count) => (StatusCode::OK, Json(count)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn create_contact(Json(payload): Json<Contact>) -> Response {
    match Contact::insert(&payload).await {
        Ok(id) => (StatusCode::OK, Json(id)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_contact_by_id(Path(id): Path<String>) -> Response {
    match Contact::delete_by_id(&id).await {
        Ok(true) => (StatusCode::OK).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Contact not found").into_response(),
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

async fn sse_events(
    State(sse_manager): State<Arc<SseManager>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let rx_stream = tokio_stream::wrappers::BroadcastStream::new(sse_manager.subscribe()).map(
        |msg| match msg {
            Ok(cnversations) => {
                let timestamp = chrono::Utc::now().timestamp_millis();
                Ok(Event::default()
                    .id(timestamp.to_string())
                    .event("conversations")
                    .json_data(&cnversations)
                    .unwrap())
            }
            Err(_) => Ok(Event::default()
                .event("error")
                .comment("Failed to receive broadcast message")),
        },
    );

    Sse::new(rx_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .event(
                Event::default()
                    .event("keep-alive")
                    .id(chrono::Utc::now().timestamp_millis().to_string()),
            ),
    )
}

async fn get_conversation_unread(Path(id): Path<String>) -> Response {
    match SMS::query_unread_by_contact_id(&id).await {
        Ok(messages) => (StatusCode::OK, Json(messages)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct SmsPayload {
    modem_id: String,
    contact: Contact,
    message: String,
    new: bool,
}

#[derive(Serialize)]
pub struct ModemInfo {
    pub name: String,
    pub com_port: String,
    pub baud_rate: u32,
}
