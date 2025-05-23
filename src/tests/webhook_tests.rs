use crate::{config::WebhookConfig, db::ModemSMS, webhook::start_webhook_worker_with_concurrency};

use chrono::NaiveDateTime;
use std::{collections::HashMap, time::Duration};
use wiremock::{
    matchers::{method, path, body_json, header, query_param},
    Mock, MockServer, ResponseTemplate,
};
use serde_json::json;

fn create_test_sms() -> ModemSMS {
    ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content".to_string(),
        device: "test_device".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        send: false,
    }
}

fn create_simple_webhook_config(url: &str) -> WebhookConfig {
    WebhookConfig {
        url: url.to_string(),
        method: "POST".to_string(),
        headers: Some({
            let mut map = HashMap::new();
            map.insert("Content-Type".to_string(), "application/json".to_string());
            map
        }),
        body: Some(r#"{"contact":"${contact}","message":"${message}","device":"${device}"}"#.to_string()),
        url_params: None,
        timeout: Some(5),
    }
}

fn create_regex_webhook_config(url: &str) -> WebhookConfig {
    WebhookConfig {
        url: url.to_string(),
        method: "POST".to_string(),
        headers: Some({
            let mut map = HashMap::new();
            map.insert("Content-Type".to_string(), "application/json".to_string());
            map.insert("X-Phone".to_string(), "${value=contact, regex=^(\\d+)$}".to_string());
            map
        }),
        body: Some(r#"{"contact":"${contact}","extractedMsg":"${value=message, regex=Test (.+) content}"}"#.to_string()),
        url_params: Some({
            let mut map = HashMap::new();
            map.insert("phone".to_string(), "${contact}".to_string());
            map
        }),
        timeout: Some(5),
    }
}

#[tokio::test]
async fn test_simple_webhook() {
    let mock_server = MockServer::start().await;
    
    let test_sms = create_test_sms();
    let expected_body = json!({
        "contact": "13800138000",
        "message": "Test message content",
        "device": "test_device"
    });
    
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .and(body_json(&expected_body))
        .and(header("Content-Type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "success"})))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let config = create_simple_webhook_config(&webhook_url);
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);
    
    webhook_manager.send(test_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_regex_webhook() {
    let mock_server = MockServer::start().await;
    
    let test_sms = create_test_sms();
    let expected_body = json!({
        "contact": "13800138000",
        "extractedMsg": "message"
    });
    
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .and(query_param("phone", "13800138000"))
        .and(header("X-Phone", "13800138000"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let config = create_regex_webhook_config(&webhook_url);
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);
    
    webhook_manager.send(test_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_multiple_webhooks() {
    let mock_server = MockServer::start().await;
    
    let test_sms = create_test_sms();
    
    Mock::given(method("POST"))
        .and(path("/webhook1"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    Mock::given(method("POST"))
        .and(path("/webhook2"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let webhook_url1 = format!("{}/webhook1", mock_server.uri());
    let webhook_url2 = format!("{}/webhook2", mock_server.uri());
    let config1 = create_simple_webhook_config(&webhook_url1);
    let config2 = create_simple_webhook_config(&webhook_url2);
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config1, config2], 5);
    
    webhook_manager.send(test_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_webhook_concurrency_limit() {
    let mock_server = MockServer::start().await;
    
    let test_sms = create_test_sms();
    
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(500)))
        .expect(3)
        .mount(&mock_server)
        .await;
    
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(500)))
        .expect(3)
        .mount(&mock_server)
        .await;
    
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let config = create_simple_webhook_config(&webhook_url);
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 2);
    
    webhook_manager.send(test_sms.clone()).unwrap();
    webhook_manager.send(test_sms.clone()).unwrap();
    webhook_manager.send(test_sms.clone()).unwrap();
    webhook_manager.send(test_sms.clone()).unwrap();
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    mock_server.verify().await;
}

#[tokio::test]
async fn test_webhook_error_handling() {
    let mock_server = MockServer::start().await;
    
    let test_sms = create_test_sms();
    
    Mock::given(method("POST"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let webhook_url = format!("{}/error", mock_server.uri());
    let config = create_simple_webhook_config(&webhook_url);
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);
    
    webhook_manager.send(test_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    mock_server.verify().await;
    
    assert_eq!(webhook_manager.available_permits(), 5);
}

#[tokio::test]
async fn test_webhook_graceful_shutdown() {
    let mock_server = MockServer::start().await;
    
    let test_sms = create_test_sms();
    
    Mock::given(method("POST"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(1)))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let webhook_url = format!("{}/slow", mock_server.uri());
    let config = create_simple_webhook_config(&webhook_url);
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);
    
    webhook_manager.send(test_sms).unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    assert_eq!(webhook_manager.available_permits(), 4);
    
    webhook_manager.shutdown().await;
    
    mock_server.verify().await;
    
    assert_eq!(webhook_manager.available_permits(), 5);
}