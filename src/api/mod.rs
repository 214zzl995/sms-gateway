use std::{convert::Infallible, sync::Arc, time::Duration};
use fancy_regex::Regex;

use axum::{
    extract::{Path, Query, State},
    response::{sse::Event, IntoResponse, Response, Sse},
    routing::{delete, get, post, put},
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
    db::{Contact, Conversation, Sms, SimCard},
    modem::{SmsType, SignalQuality, OperatorInfo, ModemInfo as ModemModel},
    config::SmsStorage,
    ModemManagerRef,
};

fn decode_sms_center(sms_center: &str) -> String {
    // Check if it's UCS2 encoded (contains sequences like 002B, 0030, etc.)
    if sms_center.contains("002B") || sms_center.contains("0030") {
        let mut decoded = String::new();
        let chars: Vec<char> = sms_center.chars().collect();
        
        for chunk in chars.chunks(4) {
            if chunk.len() == 4 {
                let hex_str: String = chunk.iter().collect();
                if let Ok(char_code) = u16::from_str_radix(&hex_str, 16) {
                    if char_code > 0 {
                        if let Some(ch) = char::from_u32(char_code as u32) {
                            decoded.push(ch);
                        }
                    }
                }
            }
        }
        
        if !decoded.is_empty() {
            return decoded;
        }
    }
    
    sms_center.to_string()
}

fn format_memory_status(memory_status: &str) -> String {
    // Parse +CPMS: "SM",5,10,"SM",5,10,"SM",5,10 format
    if let Ok(re) = Regex::new(r#"\+CPMS:\s*"([^"]+)",(\d+),(\d+),"([^"]+)",(\d+),(\d+),"([^"]+)",(\d+),(\d+)"#) {
        if let Ok(Some(captures)) = re.captures(memory_status) {
            if let (Some(used1), Some(max1), Some(used2), Some(max2), Some(used3), Some(max3)) = (
                captures.get(2),
                captures.get(3),
                captures.get(5),
                captures.get(6),
                captures.get(8),
                captures.get(9),
            ) {
                return format!(
                    "Read: {}/{}, Write: {}/{}, Receive: {}/{}", 
                    used1.as_str(), max1.as_str(), 
                    used2.as_str(), max2.as_str(), 
                    used3.as_str(), max3.as_str()
                );
            }
        }
    }
    
    memory_status.to_string()
}

mod auth;
mod sse_manager;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Asset;

pub async fn run_api(
    modem_manager: ModemManagerRef,
    server_host: &str,
    server_port: &u16,
    username: &str,
    password: &str,
    sse_manager: Arc<SseManager>,
) -> anyhow::Result<()> {
    let api = Router::new()
        .route("/check", get(check))
        .route("/sms", get(get_sms_paginated))
        .route("/sms", post(send_sms).with_state(modem_manager.clone()))
        .route("/sms/sse", get(sse_events).with_state(sse_manager.clone()))
        // 破坏性改造: 删除所有/api/device路径，改为/api/sims
        .route(
            "/sims/info",
            get(get_all_sim_info).with_state(modem_manager.clone()),
        )
        .route(
            "/sims/{sim_id}/refresh",
            get(refresh_sim_sms).with_state(modem_manager.clone()),
        )
        .route("/contacts", get(get_contacts))
        .route("/contacts", post(create_contact))
        .route("/contacts/{id}", delete(delete_contact_by_id))
        .route("/conversation", get(get_conversation))
        .route("/conversations/{id}/unread", post(get_conversation_unread))
        .route("/sim-cards", get(get_all_sim_cards)) // 保留用于管理
        .route(
            "/sims/{sim_id}/info",
            get(get_enhanced_sim_info).with_state(modem_manager.clone()),
        )
        .route("/sim-cards/{sim_id}/alias", put(update_sim_alias).with_state(modem_manager.clone()))
        .route("/sim-cards/{sim_id}/phone", put(update_sim_phone).with_state(modem_manager.clone()))
        .route(
            "/sims/{sim_id}/storage",
            get(get_sms_storage_status).with_state(modem_manager.clone()),
        )
        .route(
            "/sims/{sim_id}/storage",
            put(set_sms_storage).with_state(modem_manager.clone()),
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
    data: Vec<Sms>,
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
            Sms::paginate_by_contact_id(contact_id, query.page, query.per_page).await
        }
        None => Sms::paginate(query.page, query.per_page).await,
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
    State(modem_manager): State<ModemManagerRef>,
    Json(mut payload): Json<SmsPayload>,
) -> impl IntoResponse {
    if payload.new {
        payload.contact.find_or_create().await.unwrap();
    }

    match modem_manager.send_sms(&payload.sim_id, &payload.contact, &payload.message).await {
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
    }
}

async fn get_all_sim_info(State(modem_manager): State<ModemManagerRef>) -> Response {
    use futures::future::join_all;
    use tokio::time::{timeout, Duration};

    fn to_data_error<T, E: ToString>(result: Result<T, E>) -> (Option<T>, Option<String>) {
        match result {
            Ok(data) => (Some(data), None),
            Err(e) => (None, Some(e.to_string())),
        }
    }

    let sim_ids = modem_manager.get_sim_ids();

    // 并发获取所有调制解调器信息，带超时控制
    let sim_ids = sim_ids.await;
    let modem_futures: Vec<_> = sim_ids.iter().map(|sim_id| {
        let sim_id = sim_id.clone();
        let modem_manager = modem_manager.clone();
        async move {
            log::debug!("Getting SIM details for: {}", sim_id);
            
            // 并发执行所有AT命令，每个都有超时保护
            let signal_future = timeout(Duration::from_secs(5), modem_manager.get_signal_quality(&sim_id));
            let network_future = timeout(Duration::from_secs(5), modem_manager.check_network_registration(&sim_id));
            let operator_future = timeout(Duration::from_secs(5), modem_manager.check_operator(&sim_id));
            let model_future = timeout(Duration::from_secs(5), modem_manager.get_modem_model(&sim_id));
            let sms_center_future = timeout(Duration::from_secs(5), modem_manager.get_sms_center(&sim_id));
            let sim_status_future = timeout(Duration::from_secs(5), modem_manager.get_sim_status(&sim_id));
            let memory_status_future = timeout(Duration::from_secs(5), modem_manager.get_memory_status(&sim_id));

            let (signal_result, network_result, operator_result, model_result, sms_center_result, sim_status_result, memory_status_result) = tokio::join!(
                signal_future,
                network_future, 
                operator_future,
                model_future,
                sms_center_future,
                sim_status_future,
                memory_status_future
            );

            let (signal_data, signal_error) = to_data_error(signal_result.unwrap_or_else(|_| Err(anyhow::anyhow!("Signal quality timeout"))));
            let (network_data, network_error) = to_data_error(network_result.unwrap_or_else(|_| Err(anyhow::anyhow!("Network registration timeout"))));
            let (operator_data, operator_error) = to_data_error(operator_result.unwrap_or_else(|_| Err(anyhow::anyhow!("Operator info timeout"))));
            let (model_data, model_error) = to_data_error(model_result.unwrap_or_else(|_| Err(anyhow::anyhow!("Model info timeout"))));
            let (sms_center_data, sms_center_error) = to_data_error(sms_center_result.unwrap_or_else(|_| Err(anyhow::anyhow!("SMS center timeout"))));
            let (sim_status_data, sim_status_error) = to_data_error(sim_status_result.unwrap_or_else(|_| Err(anyhow::anyhow!("SIM status timeout"))));
            let (memory_status_data, memory_status_error) = to_data_error(memory_status_result.unwrap_or_else(|_| Err(anyhow::anyhow!("Memory status timeout"))));

            let sim_data = modem_manager.get_sim_card_cached(&sim_id).await;

            log::debug!("All AT commands completed for {}", sim_id);

            (sim_id, signal_data, signal_error, network_data, network_error, 
             operator_data, operator_error, model_data, model_error, sim_data,
             sms_center_data, sms_center_error, sim_status_data, sim_status_error,
             memory_status_data, memory_status_error)
        }
    }).collect();

    let modem_results = join_all(modem_futures).await;

    // 构建响应数据
    let mut details = Vec::new();
    for (sim_id, signal_data, _signal_error, _network_data, _network_error, 
         operator_data, _operator_error, model_data, _model_error, sim_data,
         sms_center_data, _sms_center_error, sim_status_data, _sim_status_error,
         memory_status_data, _memory_status_error) in modem_results {
        
        let (sim_data, _sim_error): (Option<SimCard>, Option<String>) = (sim_data, None);

        // Get the SIM card effective alias for display
        let _display_name = if let Some(ref sim) = sim_data {
            sim.get_effective_alias()
        } else {
            format!("SIM {}", sim_id)
        };

        // Get modem info for com_port and baud_rate
        let (com_port, baud_rate) = match modem_manager.get_modem(&sim_id).await { Some(modem) => {
            (modem.com_port.clone(), modem.baud_rate)
        } _ => {
            ("N/A".to_string(), 0)
        }};

        details.push(json!({
            "sim_id": sim_id,
            "name": sim_id.clone(),
            "com_port": com_port,
            "baud_rate": baud_rate,
            "signal_quality": signal_data,
            "operator_info": operator_data, 
            "model_info": model_data,
            "sms_center": sms_center_data.as_ref().and_then(|s| s.as_ref()).map(|s| decode_sms_center(s)),
            "sim_status": sim_status_data,
            "memory_status": memory_status_data.as_ref().and_then(|s| s.as_ref()).map(|s| format_memory_status(s))
        }));
    }

    (StatusCode::OK, Json(details)).into_response()
}

async fn refresh_sim_sms(Path(sim_id): Path<String>, State(modem_manager): State<ModemManagerRef>) -> Response {
    match modem_manager.read_sms_sync_insert(&sim_id, SmsType::RecUnread).await {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(err) => (StatusCode::BAD_GATEWAY, err.to_string()).into_response(),
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
    match Sms::query_unread_by_contact_id(&id).await {
        Ok(messages) => (StatusCode::OK, Json(messages)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct SmsPayload {
    sim_id: String,
    contact: Contact,
    message: String,
    new: bool,
}

#[derive(Serialize)]
pub struct EnhancedModemInfo {
    pub name: String,
    pub com_port: String,
    pub baud_rate: u32,
    pub signal_quality: Option<SignalQuality>,
    pub operator_info: Option<OperatorInfo>,
    pub model_info: Option<ModemModel>,
    pub sms_center: Option<String>,
    pub sim_status: Option<String>,
    pub memory_status: Option<String>,
}

#[derive(Serialize)]
#[allow(dead_code)]
pub struct ModemInfo {
    pub name: String,
    pub com_port: String,
    pub baud_rate: u32,
}

async fn get_all_sim_cards() -> Response {
    match SimCard::query_all().await {
        Ok(sim_cards) => (StatusCode::OK, Json(sim_cards)).into_response(),
        Err(e) => {
            error!("Failed to get SIM cards: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get SIM cards: {}", e))
                .into_response()
        }
    }
}

async fn get_enhanced_sim_info(
    Path(sim_id): Path<String>,
    State(modem_manager): State<ModemManagerRef>,
) -> Response {
    match modem_manager.get_modem(&sim_id).await { Some(modem) => {
        let sms_center_raw = modem_manager.get_sms_center(&sim_id).await.ok().flatten();
        let memory_status_raw = modem_manager.get_memory_status(&sim_id).await.ok().flatten();
        
        let enhanced_info = EnhancedModemInfo {
            name: sim_id.clone(),
            com_port: modem.com_port.clone(),
            baud_rate: modem.baud_rate,
            signal_quality: modem_manager.get_signal_quality(&sim_id).await.ok().flatten(),
            operator_info: modem_manager.check_operator(&sim_id).await.ok().flatten(),
            model_info: modem_manager.get_modem_model(&sim_id).await.ok().flatten(),
            sms_center: sms_center_raw.as_ref().map(|s| decode_sms_center(s)),
            sim_status: modem_manager.get_sim_status(&sim_id).await.ok().flatten(),
            memory_status: memory_status_raw.as_ref().map(|s| format_memory_status(s)),
        };
        
        (StatusCode::OK, Json(enhanced_info)).into_response()
    } _ => {
        (StatusCode::NOT_FOUND, "SIM not found").into_response()
    }}
}

#[derive(serde::Deserialize)]
pub struct UpdateAliasRequest {
    alias: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdatePhoneRequest {
    phone_number: Option<String>,
}

async fn update_sim_alias(
    Path(sim_id): Path<String>,
    State(modem_manager): State<ModemManagerRef>,
    Json(request): Json<UpdateAliasRequest>,
) -> Response {
    match SimCard::query_all().await {
        Ok(sim_cards) => {
            if let Some(mut sim_card) = sim_cards.into_iter().find(|s| s.id == sim_id) {
                match sim_card.update_alias(request.alias.clone()).await {
                    Ok(_) => {
                        // Update cache
                        modem_manager.update_sim_cache(sim_card.clone()).await;
                        (StatusCode::OK, Json(sim_card)).into_response()
                    },
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to update alias: {}", e),
                    ).into_response(),
                }
            } else {
                (StatusCode::NOT_FOUND, "SIM card not found").into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query SIM cards: {}", e),
        ).into_response(),
    }
}

async fn update_sim_phone(
    Path(sim_id): Path<String>,
    State(modem_manager): State<ModemManagerRef>,
    Json(request): Json<UpdatePhoneRequest>,
) -> Response {
    match SimCard::query_all().await {
        Ok(sim_cards) => {
            if let Some(mut sim_card) = sim_cards.into_iter().find(|s| s.id == sim_id) {
                match sim_card.update_phone_number(request.phone_number.clone()).await {
                    Ok(_) => {
                        // Update cache
                        modem_manager.update_sim_cache(sim_card.clone()).await;
                        (StatusCode::OK, Json(sim_card)).into_response()
                    },
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to update phone number: {}", e),
                    ).into_response(),
                }
            } else {
                (StatusCode::NOT_FOUND, "SIM card not found").into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query SIM cards: {}", e),
        ).into_response(),
    }
}

// DELETE: refresh_alias_mapping_api - 不再需要，因为直接使用SIM ID作为键
// async fn refresh_alias_mapping_api() {...}

#[derive(Deserialize)]
struct SmsStorageRequest {
    storage: SmsStorage,
}

async fn get_sms_storage_status(
    Path(sim_id): Path<String>,
    State(modem_manager): State<ModemManagerRef>,
) -> Response {
    match modem_manager.get_sms_storage_status(&sim_id).await {
        Ok(Some(status)) => (StatusCode::OK, Json(json!({"status": status}))).into_response(),
        Ok(None) => (StatusCode::OK, Json(json!({"status": "Unknown"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to get SMS storage status: {}", e)}))).into_response(),
    }
}

async fn set_sms_storage(
    Path(sim_id): Path<String>,
    State(modem_manager): State<ModemManagerRef>,
    Json(request): Json<SmsStorageRequest>,
) -> Response {
    match modem_manager.set_sms_storage(&sim_id, request.storage).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "SMS storage location updated successfully"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to set SMS storage: {}", e)}))).into_response(),
    }
}
