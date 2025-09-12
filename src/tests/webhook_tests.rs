use crate::{config::WebhookConfig, db::ModemSMS, webhook::start_webhook_worker_with_concurrency};

use chrono::{NaiveDateTime};
use serde_json::json;
use std::time::Duration;
use wiremock::{
    matchers::{body_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn create_test_sms() -> ModemSMS {
    ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    }
}

fn create_simple_webhook_config(url: &str) -> WebhookConfig {
    let toml = format!(
        r#"
url = "{url}"
method = "POST"
timeout = 5
body = '''{{"contact":"@contact@","message":"@message@","sim":"@sim@"}}'''

[headers]
Content-Type = "application/json"

"#,
        url = url
    );

    toml::from_str(&toml).expect("Failed to parse WebhookConfig from toml")
}

fn create_regex_webhook_config(url: &str) -> WebhookConfig {
    let toml = format!(
        r#"
url = "{url}"
method = "POST"
timeout = 5
body = '''{{"contact":"@contact@","extracted_code":"@message::\[(\d+)\]::1@","named_match":"@message::prefix: (?P<code>\d+)::code@"}}'''

[headers]
Content-Type = "application/json"

[url_params]
id = "@sim@"
code = '''@message::\[(\d+)\]@'''

"#,
        url = url
    );

    toml::from_str(&toml).expect("Failed to parse WebhookConfig from toml")
}

fn create_filtered_webhook_config(url: &str) -> WebhookConfig {
    let toml = format!(
        r#"
url = "{url}"
method = "POST"
timeout = 5
body = '''{{"contact":"@contact@","message":"@message@","sim":"@sim@"}}'''

contact_filter = ["13800138000", "13900139000"]
sim_filter = ["SIM-m_id"]
include_self_sent = false

[headers]
Content-Type = "application/json"

[message_filter]
contains = ["Test", "message"]
not_contains = ["ignore", "skip"]
regex = ".*content$"

[time_filter]
start_time = "08:00"
end_time = "18:00"
days_of_week = [1, 2, 3, 4, 5]  

"#,
        url = url
    );

    toml::from_str(&toml).expect("Failed to parse WebhookConfig from toml")
}

#[tokio::test]
async fn test_simple_webhook() {
    let mock_server = MockServer::start().await;

    let test_sms = create_test_sms();
    let expected_body = json!({
        "contact": "13800138000",
        "message": "Test message content",
        "sim": "SIM-m_id"  // This will be the alias now
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
async fn test_regex_extraction_webhook() {
    let mock_server = MockServer::start().await;

    let mut test_sms = create_test_sms();
    test_sms.message = "prefix: 12345 [67890] suffix".to_string();

    let expected_body = json!({
        "contact": "13800138000",
        "extracted_code": "67890",
        "named_match": "12345"
    });

    Mock::given(method("POST"))
        .and(path("/regex-webhook"))
        .and(body_json(&expected_body))
        .and(header("Content-Type", "application/json"))
        .and(wiremock::matchers::query_param("id", "SIM-m_id"))
        .and(wiremock::matchers::query_param("code", "67890"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "success"})))
        .expect(1)
        .mount(&mock_server)
        .await;

    let webhook_url = format!("{}/regex-webhook", mock_server.uri());
    let config = create_regex_webhook_config(&webhook_url);

    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);

    webhook_manager.send(test_sms).unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    mock_server.verify().await;
}

#[tokio::test]
async fn test_contact_filter() {
    let mock_server = MockServer::start().await;
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let config = create_filtered_webhook_config(&webhook_url);

    let matching_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };
    let non_matching_sms = ModemSMS {
        contact: "13700137000".to_string(),
        message: "Test message content".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)  
        .mount(&mock_server)
        .await;

    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);

    webhook_manager.send(matching_sms).unwrap();
    webhook_manager.send(non_matching_sms).unwrap();  

    tokio::time::sleep(Duration::from_secs(1)).await;

    mock_server.verify().await;
}

#[tokio::test]
async fn test_device_filter() {
    let mock_server = MockServer::start().await;
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let config = create_filtered_webhook_config(&webhook_url);

    let matching_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    let non_matching_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "different_device".to_string(),
    };

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);

    webhook_manager.send(matching_sms).unwrap();
    webhook_manager.send(non_matching_sms).unwrap(); 

    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_message_filter() {
    let mock_server = MockServer::start().await;
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let config = create_filtered_webhook_config(&webhook_url);

    let matching_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content".to_string(), 
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    let contains_ignore_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message ignore content".to_string(), 
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    let regex_mismatch_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message something".to_string(), 
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);

    webhook_manager.send(matching_sms).unwrap();
    webhook_manager.send(contains_ignore_sms).unwrap();  
    webhook_manager.send(regex_mismatch_sms).unwrap();  

    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_time_filter() {
    let mock_server = MockServer::start().await;
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let config = create_filtered_webhook_config(&webhook_url);

    let working_hours_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(), 
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    let off_hours_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 20:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    let weekend_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-24 10:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(), 
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1) 
        .mount(&mock_server)
        .await;

    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);

    webhook_manager.send(working_hours_sms).unwrap();
    webhook_manager.send(off_hours_sms).unwrap(); 
    webhook_manager.send(weekend_sms).unwrap(); 

    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_include_self_sent_enabled() {
    let mock_server = MockServer::start().await;
    let webhook_url = format!("{}/webhook", mock_server.uri());
    
    let toml = format!(
        r#"
url = "{url}"
method = "POST"
timeout = 5
body = '''{{"contact":"@contact@","message":"@message@","sim":"@sim@","send":"@send@"}}'''

contact_filter = ["13800138000", "13900139000"]
sim_filter = ["SIM-m_id"]
include_self_sent = true

[headers]
Content-Type = "application/json"

"#,
        url = webhook_url
    );
    let config: WebhookConfig = toml::from_str(&toml).expect("Failed to parse WebhookConfig");

    let self_sent_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content from sent".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: true,
        sim_id: "test_sim_id".to_string(),
    };

    let received_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content from received".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:01:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&mock_server)
        .await;

    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);

    webhook_manager.send(self_sent_sms).unwrap();
    webhook_manager.send(received_sms).unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_include_self_sent_disabled() {
    let mock_server = MockServer::start().await;
    let webhook_url = format!("{}/webhook", mock_server.uri());
    
    let toml = format!(
        r#"
url = "{url}"
method = "POST"
timeout = 5
body = '''{{"contact":"@contact@","message":"@message@","sim":"@sim@","send":"@send@"}}'''

contact_filter = ["13800138000", "13900139000"]
sim_filter = ["SIM-m_id"]
include_self_sent = false

[headers]
Content-Type = "application/json"

"#,
        url = webhook_url
    );
    let config: WebhookConfig = toml::from_str(&toml).expect("Failed to parse WebhookConfig");

    let self_sent_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content from sent".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: true,
        sim_id: "test_sim_id".to_string(),
    };

    let received_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content from received".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:01:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .and(body_json(&json!({
            "contact": "13800138000",
            "message": "Test message content from received",
            "sim": "SIM-m_id",
            "send": "false"
        })))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);

    webhook_manager.send(self_sent_sms).unwrap(); 
    webhook_manager.send(received_sms).unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_include_self_sent_default() {
    let mock_server = MockServer::start().await;
    let webhook_url = format!("{}/webhook", mock_server.uri());
    
    let toml = format!(
        r#"
url = "{url}"
method = "POST"
timeout = 5
body = '''{{"contact":"@contact@","message":"@message@","sim":"@sim@","send":"@send@"}}'''

contact_filter = ["13800138000", "13900139000"]
sim_filter = ["SIM-m_id"]

[headers]
Content-Type = "application/json"

"#,
        url = webhook_url
    );
    let config: WebhookConfig = toml::from_str(&toml).expect("Failed to parse WebhookConfig");

    let self_sent_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content from sent".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: true,
        sim_id: "test_sim_id".to_string(),
    };

    let received_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message content from received".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:01:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .and(body_json(&json!({
            "contact": "13800138000",
            "message": "Test message content from received",
            "sim": "SIM-m_id",
            "send": "false"
        })))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);

    webhook_manager.send(self_sent_sms).unwrap(); 
    webhook_manager.send(received_sms).unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_server.verify().await;
}

#[tokio::test]
async fn test_url_encoding_fix() {
    let mock_server = MockServer::start().await;

    let test_sms = ModemSMS {
        contact: "13800138000".to_string(),
        message: "Test message with special chars: 你好世界 & spaces".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2025-05-23 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        send: false,
        sim_id: "test_sim_id".to_string(),
    };
    let expected_body = json!({
        "contact": "13800138000",
        "message": "Test message with special chars: 你好世界 & spaces",
        "sim": "SIM-m_id"
    });

    Mock::given(method("POST"))
        .and(path("/webhook/SIM-m_id"))  
        .and(body_json(&expected_body))
        .and(header("Content-Type", "application/json"))
        .and(wiremock::matchers::query_param("sim", "SIM-m_id"))  // 参数值应该被编码
        .and(wiremock::matchers::query_param("message", "Test%20message%20with%20special%20chars%3A%20%E4%BD%A0%E5%A5%BD%E4%B8%96%E7%95%8C%20%26%20spaces"))  // 参数值应该被编码
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "success"})))
        .expect(1)
        .mount(&mock_server)
        .await;

    let webhook_url = format!("{}/webhook/@sim@", mock_server.uri());
    let toml = format!(
        r#"
url = "{url}"
method = "POST"
timeout = 5
body = '''{{"contact":"@contact@","message":"@message@","sim":"@sim@"}}'''

[headers]
Content-Type = "application/json"

[url_params]
sim = "@sim@"
message = "@message@"

"#,
        url = webhook_url
    );

    let config: WebhookConfig = toml::from_str(&toml).expect("Failed to parse WebhookConfig");
    let webhook_manager = start_webhook_worker_with_concurrency(vec![config], 5);

    webhook_manager.send(test_sms).unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    mock_server.verify().await;
}

