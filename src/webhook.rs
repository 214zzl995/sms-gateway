use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, sync::LazyLock};

use crate::config::{MessageFilter, TimeFilter, WebhookConfig};
use crate::db::ModemSMS;
use chrono::{Datelike, NaiveDateTime};
use log::{debug, error, info, warn};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::{mpsc, Semaphore};

static RE_PLACEHOLDER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{(?P<content>.+?)\}").unwrap());
static RE_REGEX_PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"value\s*=\s*(?P<value_key>\w+)\s*,\s*regex\s*=\s*(?P<regex_pattern>.+)").unwrap()
});

#[derive(Clone)]
enum PlaceholderType {
    Simple(String),
    Regex(String, Regex),
}

#[derive(Clone)]
struct Placeholder {
    placeholder_type: PlaceholderType,
}

#[derive(Clone)]
struct PreprocessedTemplate {
    parts: Vec<TemplatePart>,
}

#[derive(Clone)]
enum TemplatePart {
    StaticText(String),
    Placeholder(Placeholder),
}

#[derive(Clone)]
struct PreprocessedWebhookConfig {
    url: PreprocessedTemplate,
    method: String,
    headers: Option<HashMap<String, PreprocessedTemplate>>,
    body: Option<PreprocessedTemplate>,
    url_params: Option<HashMap<String, PreprocessedTemplate>>,
    timeout: Option<u64>,
    
    // Filters
    contact_filter: Option<Vec<String>>,
    device_filter: Option<Vec<String>>,
    time_filter: Option<TimeFilter>,
    message_filter: Option<MessageFilter>,
    self_sent_only: Option<bool>,
}

impl PreprocessedTemplate {
    fn new(template: &str) -> Self {
        let mut parts = Vec::new();
        let mut last_end = 0;
        
        for caps in RE_PLACEHOLDER.captures_iter(template) {
            let full_match = caps.get(0).unwrap();
            let content = caps.name("content").unwrap().as_str().trim();
            
            if full_match.start() > last_end {
                let static_text = &template[last_end..full_match.start()];
                parts.push(TemplatePart::StaticText(static_text.to_string()));
            }
            
            let placeholder_type = if let Some(regex_caps) = RE_REGEX_PLACEHOLDER.captures(content) {
                let value_key = regex_caps.name("value_key").unwrap().as_str().to_string();
                let regex_pattern = regex_caps.name("regex_pattern").unwrap().as_str();
                
                match Regex::new(regex_pattern) {
                    Ok(re) => PlaceholderType::Regex(value_key, re),
                    Err(_) => {
                        warn!("Invalid regex pattern: {}", regex_pattern);
                        PlaceholderType::Simple("error".to_string())
                    }
                }
            } else {
                PlaceholderType::Simple(content.to_string())
            };
            
            parts.push(TemplatePart::Placeholder(Placeholder {
                placeholder_type,
            }));
            
            last_end = full_match.end();
        }
        
        if last_end < template.len() {
            parts.push(TemplatePart::StaticText(template[last_end..].to_string()));
        }
        
        PreprocessedTemplate {
            parts,
        }
    }
    
    fn apply(&self, msg: &ModemSMS) -> String {
        let mut result = String::new();
        
        for part in &self.parts {
            match part {
                TemplatePart::StaticText(text) => {
                    result.push_str(text);
                },
                TemplatePart::Placeholder(placeholder) => {
                    match &placeholder.placeholder_type {
                        PlaceholderType::Simple(key) => {
                            let value = match key.as_str() {
                                "contact" => msg.contact.clone(),
                                "message" => msg.message.clone(),
                                "device" => msg.device.clone(),
                                "timestamp" => msg.timestamp.to_string(),
                                _ => {
                                    warn!("Unknown simple placeholder: {}", key);
                                    String::new()
                                },
                            };
                            result.push_str(&value);
                        },
                        PlaceholderType::Regex(value_key, regex) => {
                            let value_to_match = match value_key.as_str() {
                                "contact" => msg.contact.clone(),
                                "message" => msg.message.clone(),
                                "device" => msg.device.clone(),
                                "timestamp" => msg.timestamp.to_string(),
                                _ => {
                                    warn!("Unknown placeholder key: {}", value_key);
                                    String::new()
                                },
                            };
                            
                            if let Some(matched) = regex.captures(&value_to_match) {
                                if let Some(first_capture) = matched.get(1) {
                                    result.push_str(first_capture.as_str());
                                }
                            }
                        },
                    }
                },
            }
        }
        
        result
    }
}

impl PreprocessedWebhookConfig {
    fn from_config(config: &WebhookConfig) -> Self {
        let mut headers_map = None;
        if let Some(h) = &config.headers {
            let mut processed_headers = HashMap::new();
            for (key, value) in h {
                processed_headers.insert(key.clone(), PreprocessedTemplate::new(value));
            }
            headers_map = Some(processed_headers);
        }
        
        let mut url_params_map = None;
        if let Some(params) = &config.url_params {
            let mut processed_params = HashMap::new();
            for (key, value) in params {
                processed_params.insert(key.clone(), PreprocessedTemplate::new(value));
            }
            url_params_map = Some(processed_params);
        }
        
        PreprocessedWebhookConfig {
            url: PreprocessedTemplate::new(&config.url),
            method: config.method.clone(),
            headers: headers_map,
            body: config.body.as_ref().map(|b| PreprocessedTemplate::new(b)),
            url_params: url_params_map,
            timeout: config.timeout,
            contact_filter: config.contact_filter.clone(),
            device_filter: config.device_filter.clone(),
            time_filter: config.time_filter.clone(),
            message_filter: config.message_filter.clone(),
            self_sent_only: config.self_sent_only,
        }
    }
    
    // Check if a message passes all the filters
    fn passes_filters(&self, msg: &ModemSMS) -> bool {
        // Contact filter
        if let Some(contacts) = &self.contact_filter {
            if !contacts.is_empty() && !contacts.contains(&msg.contact) {
                return false;
            }
        }
        
        // Device filter
        if let Some(devices) = &self.device_filter {
            if !devices.is_empty() && !devices.contains(&msg.device) {
                return false;
            }
        }
        
        // Time filter
        if let Some(time_filter) = &self.time_filter {
            if !self.passes_time_filter(time_filter, &msg.timestamp) {
                return false;
            }
        }
        
        // Message filter
        if let Some(message_filter) = &self.message_filter {
            if !self.passes_message_filter(message_filter, &msg.message) {
                return false;
            }
        }
        
        // Self-sent filter
        if let Some(self_sent_only) = &self.self_sent_only {
            // If self_sent_only is true, only allow messages with send=true (sent by user)
            // If self_sent_only is false, only allow messages with send=false (received messages)
            if (*self_sent_only && !msg.send) || (!*self_sent_only && msg.send) {
                return false;
            }
        }
        
        true
    }
    
    // Check if a message passes the time filter
    fn passes_time_filter(&self, time_filter: &TimeFilter, timestamp: &NaiveDateTime) -> bool {
        // Check days of week if specified
        if let Some(days) = &time_filter.days_of_week {
            if !days.is_empty() {
                let weekday = timestamp.weekday().num_days_from_sunday() as u8;
                if !days.contains(&weekday) {
                    return false;
                }
            }
        }
        
        // Check time range if specified
        if let (Some(start), Some(end)) = (&time_filter.start_time, &time_filter.end_time) {
            if !start.is_empty() && !end.is_empty() {
                let time = timestamp.format("%H:%M").to_string();
                
                // Simple string comparison works for HH:MM format
                if time < *start || time > *end {
                    return false;
                }
            }
        }
        
        true
    }
    
    // Check if a message passes the message content filter
    fn passes_message_filter(&self, message_filter: &MessageFilter, message: &str) -> bool {
        // Check 'contains' strings
        if let Some(contains_list) = &message_filter.contains {
            if !contains_list.is_empty() && !contains_list.iter().any(|s| message.contains(s)) {
                return false;
            }
        }
        
        // Check 'not_contains' strings
        if let Some(not_contains_list) = &message_filter.not_contains {
            if !not_contains_list.is_empty() && not_contains_list.iter().any(|s| message.contains(s)) {
                return false;
            }
        }
        
        // Check regex pattern
        if let Some(pattern) = &message_filter.regex {
            if !pattern.is_empty() {
                match Regex::new(pattern) {
                    Ok(re) => {
                        if !re.is_match(message) {
                            return false;
                        }
                    },
                    Err(e) => {
                        warn!("Invalid regex pattern in message filter: {}", e);
                    }
                }
            }
        }
        
        true
    }
}

#[derive(Clone)]
pub struct WebhookManager {
    client: Client,
    configs: Arc<Vec<PreprocessedWebhookConfig>>,
    sender: mpsc::UnboundedSender<ModemSMS>,
    semaphore: Arc<Semaphore>,
    max_concurrent_requests: usize,
}

impl WebhookManager {
    pub fn new(configs: Vec<WebhookConfig>) -> Self {
        Self::new_with_concurrency(configs, 10) 
    }
    
    pub fn new_with_concurrency(configs: Vec<WebhookConfig>, max_concurrent: usize) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        let preprocessed_configs = configs.iter()
            .map(|cfg| PreprocessedWebhookConfig::from_config(cfg))
            .collect();
        
        let manager = WebhookManager {
            client: Client::new(),
            configs: Arc::new(preprocessed_configs),
            sender,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent_requests: max_concurrent,
        };
        
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            manager_clone.receiver_loop(receiver).await;
        });
        
        manager
    }

    pub fn send(&self, msg: ModemSMS) -> Result<(), mpsc::error::SendError<ModemSMS>> {
        self.sender.send(msg)
    }
    
    async fn receiver_loop(&self, mut receiver: mpsc::UnboundedReceiver<ModemSMS>) {
        while let Some(msg) = receiver.recv().await {
            debug!("Webhook worker received message: {:?}", msg);
            
            let mut tasks = Vec::new();
            for cfg in self.configs.iter() {
                let self_clone = self.clone();
                let msg_clone = msg.clone();
                let cfg_clone = cfg.clone();
                
                let task = tokio::spawn(async move {
                    self_clone.process_webhook(&cfg_clone, &msg_clone).await;
                });
                
                tasks.push(task);
            }
            
            for task in tasks {
                if let Err(e) = task.await {
                    error!("Webhook task failed: {:?}", e);
                }
            }
            
            debug!("Finished processing all webhooks for message");
        }
    }
    
    async fn process_webhook(&self, cfg: &PreprocessedWebhookConfig, msg: &ModemSMS) {
        // Check if the message passes all filters before processing
        if !cfg.passes_filters(msg) {
            debug!("Message from {} filtered out by webhook configuration", msg.contact);
            return;
        }
        
        let start_time = std::time::Instant::now();
        
        let _permit = match self.semaphore.acquire().await {
            Ok(permit) => permit,
            Err(_) => {
                error!("Failed to acquire semaphore permit for webhook");
                return;
            }
        };
        
        let client = &self.client;
        
        let url = cfg.url.apply(msg);

        let mut headers_map = reqwest::header::HeaderMap::new();
        if let Some(h) = &cfg.headers {
            for (key, template) in h {
                let value = template.apply(msg);
                if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                    if let Ok(header_value) = reqwest::header::HeaderValue::from_str(&value) {
                        headers_map.insert(header_name, header_value);
                    } else {
                        error!("Invalid header value for key {}: {}", key, value);
                    }
                } else {
                    error!("Invalid header name: {}", key);
                }
            }
        }

        let body_str = match &cfg.body {
            Some(template) => Some(template.apply(msg)),
            None => None,
        };

        let mut url_params_map = HashMap::new();
        if let Some(params) = &cfg.url_params {
            for (key, template) in params {
                url_params_map.insert(key.clone(), template.apply(msg));
            }
        }

        let timeout_duration = Duration::from_secs(cfg.timeout.unwrap_or(10)); // Default timeout 10s

        let method = match cfg.method.to_uppercase().as_str() {
            "POST" => reqwest::Method::POST,
            "GET" => reqwest::Method::GET,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            _ => {
                error!("Unsupported HTTP method: {}", cfg.method);
                return;
            }
        };

        let mut request_builder = client
            .request(method.clone(), &url)
            .headers(headers_map)
            .timeout(timeout_duration);

        if !url_params_map.is_empty() {
            request_builder = request_builder.query(&url_params_map);
        }

        if let Some(body_content) = body_str {
            if let Ok(json_body) = serde_json::from_str::<Value>(&body_content) {
                request_builder = request_builder.json(&json_body);
            } else {
                request_builder = request_builder.body(body_content);
            }
        }

        info!("Sending webhook to URL: {}, Method: {}", url, method);

        match request_builder.send().await {
            Ok(response) => {
                let elapsed = start_time.elapsed();
                info!(
                    "Webhook to {} responded with status: {} in {:?}",
                    url,
                    response.status(),
                    elapsed
                );
                
                if log::log_enabled!(log::Level::Debug) {
                    if let Ok(text) = response.text().await {
                        debug!("Webhook response body: {}", text);
                    }
                }
            }
            Err(e) => {
                let elapsed = start_time.elapsed();
                error!("Failed to send webhook to {} after {:?}: {}", url, elapsed, e);
            }
        }
    }    

    pub fn config_count(&self) -> usize {
        self.configs.len()
    }
    
    pub fn max_concurrent_requests(&self) -> usize {
        self.max_concurrent_requests
    }
    
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
    
    pub async fn shutdown(&self) {
        info!("Shutting down webhook manager...");
        
        let _permits = self.semaphore.acquire_many(self.max_concurrent_requests as u32).await;
        
        info!("Webhook manager shutdown complete");
    }
}

pub fn _start_webhook_worker(configs: Vec<WebhookConfig>) -> WebhookManager {
    WebhookManager::new(configs.to_vec())
}

pub fn start_webhook_worker_with_concurrency(configs: Vec<WebhookConfig>, max_concurrent: usize) -> WebhookManager {
    WebhookManager::new_with_concurrency(configs, max_concurrent)
}
