use anyhow::{Context, Result};
use config::{Config, File};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

/// `Settings` struct, mapping the `settings` section in the TOML config file
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server_host: String,
    pub server_port: u16,
    pub username:Option<String>,
    pub password:Option<String>,
    pub read_sms_frequency: u64,
}

/// `Device` struct, mapping the `device` section in the TOML config file
#[derive(Debug, Deserialize)]
pub struct Device {
    pub com_port: String,
    pub baud_rate: u32,
}

/// `AppConfig` struct, containing both `settings` and `devices` sections
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
