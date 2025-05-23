use anyhow::{Context, Result};
use config::{Config, File};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub url_params: Option<HashMap<String, String>>,
    pub timeout: Option<u64>,
    
    // Filters
    pub contact_filter: Option<Vec<String>>,   // List of contacts to include
    pub device_filter: Option<Vec<String>>,    // List of devices to include
    pub time_filter: Option<TimeFilter>,       // Time-based filtering
    pub message_filter: Option<MessageFilter>, // Content-based filtering
    pub self_sent_only: Option<bool>,         // If true, only process messages sent by the user
}

#[derive(Debug, Deserialize, Clone)]
pub struct TimeFilter {
    pub start_time: Option<String>,   // Format: "HH:MM" (24-hour format)
    pub end_time: Option<String>,     // Format: "HH:MM" (24-hour format)
    pub days_of_week: Option<Vec<u8>>, // 0-6, where 0 is Sunday
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessageFilter {
    pub contains: Option<Vec<String>>,      // Strings that must be in the message
    pub not_contains: Option<Vec<String>>,  // Strings that must not be in the message
    pub regex: Option<String>,              // Regular expression pattern to match
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server_host: String,
    pub server_port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub read_sms_frequency: u64,
    pub webhooks_max_concurrent: Option<usize>,
    pub webhooks: Option<Vec<WebhookConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    pub com_port: String,
    pub baud_rate: u32,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub settings: Settings,
    pub devices: HashMap<String, Device>,
}

impl AppConfig {
    /// Load configuration from a config file
    pub fn load(config_file_path: &Path) -> Result<AppConfig> {
        // Use the `config` crate to load the config file
        let config = Config::builder()
            .add_source(File::from(config_file_path))
            .build()
            .context("Failed to load config file")?;

        // Deserialize the config file into the `AppConfig` struct
        let app_config: AppConfig = config
            .try_deserialize()?;

        // Validate the configuration
        test_config(&app_config)?;

        Ok(app_config)
    }
}

/// Validate required configuration fields
fn test_config(app_config: &AppConfig) -> Result<()> {
    // Validate SETTINGS section
    if app_config.settings.server_host.trim().is_empty() {
        anyhow::bail!("Fatal: server_host is not set");
    }
    if app_config.settings.server_port == 0 {
        anyhow::bail!("Fatal: server_port is not set");
    }

    // Validate DEVICES section
    for (key, device) in &app_config.devices {
        if device.com_port.trim().is_empty() {
            anyhow::bail!("Fatal: Device {} com_port is not set", key);
        }
        if device.baud_rate == 0 {
            anyhow::bail!("Fatal: Device {} baud_rate is not set", key);
        }
    }

    Ok(())
}
