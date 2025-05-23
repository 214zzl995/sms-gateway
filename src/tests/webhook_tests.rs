use crate::{config::{MessageFilter, TimeFilter, WebhookConfig}, db::ModemSMS, webhook::start_webhook_worker_with_concurrency};

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

// Create SMS with specified parameters for testing filters
fn create_custom_sms(contact: &str, message: &str, device: &str, timestamp: &str, send: bool) -> ModemSMS {
    ModemSMS {
        contact: contact.to_string(),
        message: message.to_string(),
        device: device.to_string(),
        timestamp: NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S").unwrap(),
        send,
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
        contact_filter: None,
        device_filter: None,
        time_filter: None,
        message_filter: None,
        self_sent_only: None,
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
        contact_filter: None,
        device_filter: None,
        time_filter: None,
        message_filter: None,
        self_sent_only: None,
    }
}

// Create webhook config with contact filter
fn create_contact_filtered_webhook_config(url: &str, contacts: Vec<String>) -> WebhookConfig {
    let mut config = create_simple_webhook_config(url);
    config.contact_filter = Some(contacts);
    config
}

// Create webhook config with device filter
fn create_device_filtered_webhook_config(url: &str, devices: Vec<String>) -> WebhookConfig {
    let mut config = create_simple_webhook_config(url);
    config.device_filter = Some(devices);
    config
}

// Create webhook config with time filter
fn create_time_filtered_webhook_config(url: &str, start_time: Option<String>, end_time: Option<String>, days: Option<Vec<u8>>) -> WebhookConfig {
    let mut config = create_simple_webhook_config(url);
    config.time_filter = Some(TimeFilter {
        start_time,
        end_time,
        days_of_week: days,
    });
    config
}

// Create webhook config with message content filter
fn create_message_filtered_webhook_config(url: &str, contains: Option<Vec<String>>, not_contains: Option<Vec<String>>, regex: Option<String>) -> WebhookConfig {
    let mut config = create_simple_webhook_config(url);
    config.message_filter = Some(MessageFilter {
        contains,
        not_contains,
        regex,
    });
    config
}

// Create webhook config with self-sent filter
fn create_self_sent_filtered_webhook_config(url: &str, self_sent_only: bool) -> WebhookConfig {
    let mut config = create_simple_webhook_config(url);
    config.self_sent_only = Some(self_sent_only);
    config
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
    
    // We expect exactly 4 calls - all messages will be processed, but with concurrency limit of 2
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(500)))
        .expect(4)
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

#[tokio::test]
async fn test_contact_filter() {
    let mock_server = MockServer::start().await;
    
    // This webhook should only process messages from 13800138000
    let webhook_url1 = format!("{}/filtered", mock_server.uri());
    let config1 = create_contact_filtered_webhook_config(&webhook_url1, vec!["13800138000".to_string()]);
    
    // This webhook should only process messages from 13900139000
    let webhook_url2 = format!("{}/filtered2", mock_server.uri());
    let config2 = create_contact_filtered_webhook_config(&webhook_url2, vec!["13900139000".to_string()]);
    
    // Setup mock expectations - only webhook1 should be called with the first SMS
    Mock::given(method("POST"))
        .and(path("/filtered"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    Mock::given(method("POST"))
        .and(path("/filtered2"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)  // Should not be called
        .mount(&mock_server)
        .await;
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config1, config2], 5);
    
    // Create SMS that should only trigger the first webhook
    let test_sms = create_custom_sms(
        "13800138000", 
        "Test message content", 
        "test_device", 
        "2025-05-23 15:00:00", 
        false
    );
    
    webhook_manager.send(test_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_device_filter() {
    let mock_server = MockServer::start().await;
    
    // This webhook should only process messages from device1
    let webhook_url1 = format!("{}/filtered", mock_server.uri());
    let config1 = create_device_filtered_webhook_config(&webhook_url1, vec!["device1".to_string()]);
    
    // This webhook should only process messages from device2
    let webhook_url2 = format!("{}/filtered2", mock_server.uri());
    let config2 = create_device_filtered_webhook_config(&webhook_url2, vec!["device2".to_string()]);
    
    // Setup mock expectations - only webhook2 should be called with our test SMS
    Mock::given(method("POST"))
        .and(path("/filtered"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)  // Should not be called
        .mount(&mock_server)
        .await;
    
    Mock::given(method("POST"))
        .and(path("/filtered2"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config1, config2], 5);
    
    // Create SMS that should only trigger the second webhook
    let test_sms = create_custom_sms(
        "13800138000", 
        "Test message content", 
        "device2", 
        "2025-05-23 15:00:00", 
        false
    );
    
    webhook_manager.send(test_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_time_filter() {
    let mock_server = MockServer::start().await;
    
    // This webhook should only process messages sent between 14:00 and 16:00
    let webhook_url1 = format!("{}/filtered", mock_server.uri());
    let config1 = create_time_filtered_webhook_config(
        &webhook_url1, 
        Some("14:00".to_string()), 
        Some("16:00".to_string()), 
        None
    );
    
    // This webhook should only process messages sent between 17:00 and 19:00
    let webhook_url2 = format!("{}/filtered2", mock_server.uri());
    let config2 = create_time_filtered_webhook_config(
        &webhook_url2, 
        Some("17:00".to_string()), 
        Some("19:00".to_string()), 
        None
    );
    
    // Setup mock expectations - only webhook1 should be called with our 15:00 test SMS
    Mock::given(method("POST"))
        .and(path("/filtered"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    Mock::given(method("POST"))
        .and(path("/filtered2"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)  // Should not be called
        .mount(&mock_server)
        .await;
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config1, config2], 5);
    
    // SMS sent at 15:00 (should only trigger the first webhook)
    let test_sms = create_custom_sms(
        "13800138000", 
        "Test message content", 
        "test_device", 
        "2025-05-23 15:00:00", 
        false
    );
    
    webhook_manager.send(test_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_message_filter() {
    let mock_server = MockServer::start().await;
    
    // This webhook should only process messages containing "important"
    let webhook_url1 = format!("{}/filtered", mock_server.uri());
    let config1 = create_message_filtered_webhook_config(
        &webhook_url1, 
        Some(vec!["important".to_string()]), 
        None, 
        None
    );
    
    // This webhook should only process messages NOT containing "ignore"
    let webhook_url2 = format!("{}/filtered2", mock_server.uri());
    let config2 = create_message_filtered_webhook_config(
        &webhook_url2, 
        None, 
        Some(vec!["ignore".to_string()]), 
        None
    );
    
    // Setup mock expectations
    Mock::given(method("POST"))
        .and(path("/filtered"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    Mock::given(method("POST"))
        .and(path("/filtered2"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config1, config2], 5);
    
    // SMS containing "important" but not "ignore" (should trigger both webhooks)
    let test_sms = create_custom_sms(
        "13800138000", 
        "This is an important message", 
        "test_device", 
        "2025-05-23 15:00:00", 
        false
    );
    
    webhook_manager.send(test_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_self_sent_filter() {
    let mock_server = MockServer::start().await;
    
    // This webhook should only process messages sent by the user
    let webhook_url1 = format!("{}/filtered", mock_server.uri());
    let config1 = create_self_sent_filtered_webhook_config(&webhook_url1, true);
    
    // This webhook should only process messages received from others
    let webhook_url2 = format!("{}/filtered2", mock_server.uri());
    let config2 = create_self_sent_filtered_webhook_config(&webhook_url2, false);
    
    // Setup mock expectations - only webhook1 should be called with a self-sent SMS
    Mock::given(method("POST"))
        .and(path("/filtered"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    Mock::given(method("POST"))
        .and(path("/filtered2"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)  // Should not be called
        .mount(&mock_server)
        .await;
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config1, config2], 5);
    
    // Create a self-sent SMS (send = true)
    let test_sms = create_custom_sms(
        "13800138000", 
        "This is a message I sent myself", 
        "test_device", 
        "2025-05-23 15:00:00", 
        true  // Self-sent
    );
    
    webhook_manager.send(test_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_combined_filters() {
    let mock_server = MockServer::start().await;
    
    // This webhook has multiple filters that all need to match
    let webhook_url = format!("{}/filtered", mock_server.uri());
    let mut config = create_simple_webhook_config(&webhook_url);
    
    // Set multiple filters
    config.contact_filter = Some(vec!["13800138000".to_string()]);
    config.device_filter = Some(vec!["test_device".to_string()]);
    config.message_filter = Some(MessageFilter {
        contains: Some(vec!["urgent".to_string()]),
        not_contains: None,
        regex: None,
    });
    config.self_sent_only = Some(false);
    
    // Setup mock expectations
    Mock::given(method("POST"))
        .and(path("/filtered"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);
    
    // This SMS should match all filter criteria
    let matching_sms = create_custom_sms(
        "13800138000", 
        "This is an urgent message", 
        "test_device", 
        "2025-05-23 15:00:00", 
        false
    );
    
    // This SMS should not match the message filter
    let non_matching_sms = create_custom_sms(
        "13800138000", 
        "This is a regular message", 
        "test_device", 
        "2025-05-23 15:00:00", 
        false
    );
    
    // Send both messages, but only the matching one should trigger the webhook
    webhook_manager.send(non_matching_sms).unwrap();
    webhook_manager.send(matching_sms).unwrap();
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}