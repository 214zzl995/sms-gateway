use std::sync::Arc;

use crate::config::{MessageFilter, SegmentName, TemplateSegment, TimeFilter, WebhookConfig};
use crate::db::ModemSMS;
use chrono::{Datelike, NaiveDateTime};
use log::{debug, error, info};
use reqwest::Client;
use tokio::sync::{mpsc, Semaphore};

pub fn apply_template_segments(segments: &[TemplateSegment], msg: &ModemSMS) -> String {
    let mut result = String::new();

    for segment in segments {
        match segment {
            TemplateSegment::Fixed(text) => {
                result.push_str(text);
            }
            TemplateSegment::Placeholder(placeholder) => {
                let value = match &placeholder.name {
                    SegmentName::Contact => msg.contact.clone(),
                    SegmentName::Message => msg.message.clone(),
                    SegmentName::Device => msg.device.clone(),
                    SegmentName::Timestamp => msg.timestamp.to_string(),
                    SegmentName::Send => msg.send.to_string(),
                };

                if let Some(regex) = &placeholder.regex {
                    if let Some(caps) = regex.captures(&value) {
                        let extracted = if let Some(name) = &placeholder.regex_name {
                            caps.name(name).map(|m| m.as_str().to_string())
                        } else if let Some(index) = placeholder.regex_index {
                            caps.get(index).map(|m| m.as_str().to_string())
                        } else {
                            caps.get(1).map(|m| m.as_str().to_string())
                        };

                        if let Some(text) = extracted {
                            result.push_str(&text);
                        }
                    }
                } else {
                    result.push_str(&value);
                }
            }
        }
    }

    result
}

#[derive(Clone)]
pub struct WebhookManager {
    client: Client,
    pub(crate) configs: Arc<Vec<WebhookConfig>>,
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

        let manager = WebhookManager {
            client: Client::new(),
            configs: Arc::new(configs),
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

    pub(crate) fn passes_filters(&self, config: &WebhookConfig, msg: &ModemSMS) -> bool {
        if let Some(contacts) = &config.contact_filter {
            if !contacts.is_empty() && !contacts.contains(&msg.contact) {
                return false;
            }
        }

        if let Some(devices) = &config.device_filter {
            if !devices.is_empty() && !devices.contains(&msg.device) {
                return false;
            }
        }

        if let Some(time_filter) = &config.time_filter {
            if !self.passes_time_filter(time_filter, &msg.timestamp) {
                return false;
            }
        }

        if let Some(message_filter) = &config.message_filter {
            if !self.passes_message_filter(message_filter, &msg.message) {
                return false;
            }
        }

        let include_self_sent = config.include_self_sent.unwrap_or(false);
        if msg.send && !include_self_sent {
            return false;
        }

        true
    }

    fn passes_time_filter(&self, time_filter: &TimeFilter, timestamp: &NaiveDateTime) -> bool {
        if let Some(days) = &time_filter.days_of_week {
            if !days.is_empty() {
                let weekday = timestamp.weekday();
                if !days.contains(&weekday) {
                    return false;
                }
            }
        }

        if let (Some(start), Some(end)) = (&time_filter.start_time, &time_filter.end_time) {
            let time = timestamp.time();
            if time < *start || time > *end {
                return false;
            }
        }

        true
    }

    fn passes_message_filter(&self, message_filter: &MessageFilter, message: &str) -> bool {
        if let Some(contains_list) = &message_filter.contains {
            if !contains_list.is_empty() && !contains_list.iter().any(|s| message.contains(s)) {
                return false;
            }
        }

        if let Some(not_contains_list) = &message_filter.not_contains {
            if !not_contains_list.is_empty()
                && not_contains_list.iter().any(|s| message.contains(s))
            {
                return false;
            }
        }

        if let Some(re) = &message_filter.regex {
            if !re.is_match(message) {
                return false;
            }
        }

        true
    }

    async fn process_webhook(&self, cfg: &WebhookConfig, msg: &ModemSMS) {
        if !self.passes_filters(cfg, msg) {
            debug!(
                "Message from {} filtered out by webhook configuration",
                msg.contact
            );
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

        let url = apply_template_segments(&cfg.url, msg);

        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(h) = &cfg.headers {
            for (key, segments) in h {
                let value = apply_template_segments(segments, msg);
                if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                    if let Ok(header_value) = reqwest::header::HeaderValue::from_str(&value) {
                        headers.insert(header_name, header_value);
                    } else {
                        error!("Invalid header value for key {}: {}", key, value);
                    }
                } else {
                    error!("Invalid header name: {}", key);
                }
            }
        }

        let body_str = if let Some(body) = &cfg.body {
            Some(apply_template_segments(body, msg))
        } else {
            None
        };

        let mut url_params = std::collections::HashMap::new();
        if let Some(params) = &cfg.url_params {
            for (key, segments) in params {
                url_params.insert(key.clone(), apply_template_segments(segments, msg));
            }
        }

        let timeout_duration = std::time::Duration::from_secs(cfg.timeout.unwrap_or(10)); 

        let method: reqwest::Method = cfg.method.clone().into();

        let mut request_builder = client
            .request(method.clone(), &url)
            .headers(headers)
            .timeout(timeout_duration);

        if !url_params.is_empty() {
            request_builder = request_builder.query(&url_params);
        }

        if let Some(body_content) = body_str {
            if let Ok(json_body) = serde_json::from_str::<serde_json::Value>(&body_content) {
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
                error!(
                    "Failed to send webhook to {} after {:?}: {}",
                    url, elapsed, e
                );
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

    #[cfg(test)]
    pub fn test_passes_filters(&self, msg: &ModemSMS) -> bool {
        if let Some(config) = self.configs.first() {
            self.passes_filters(config, msg)
        } else {
            false
        }
    }

    pub async fn shutdown(&self) {
        info!("Shutting down webhook manager...");

        let _permits = self
            .semaphore
            .acquire_many(self.max_concurrent_requests as u32)
            .await;

        info!("Webhook manager shutdown complete");
    }
}

pub fn _start_webhook_worker(configs: Vec<WebhookConfig>) -> WebhookManager {
    WebhookManager::new(configs.to_vec())
}

pub fn start_webhook_worker_with_concurrency(
    configs: Vec<WebhookConfig>,
    max_concurrent: usize,
) -> WebhookManager {
    WebhookManager::new_with_concurrency(configs, max_concurrent)
}
