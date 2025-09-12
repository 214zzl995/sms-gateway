use std::{path::PathBuf, sync::Arc};

use api::SseManager;
use db::db_init;
use flexi_logger::{
    colored_detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming,
};
use log::LevelFilter;
use modem::{SmsType, ModemManager};
use structopt::StructOpt;

mod api;
mod config;
mod db;
mod decode;
mod modem;
mod webhook;
#[cfg(test)]
mod tests;

pub type ModemManagerRef = Arc<ModemManager>;

#[tokio::main]
async fn main() {
    let param = Param::from_args();
    if let Err(err) = log_init(&param.log_path, &param.log_level) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    };
    if let Err(err) = db_init().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    #[cfg(debug_assertions)]
    let config = match config::AppConfig::load(&PathBuf::from("./config.toml")) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };
    #[cfg(not(debug_assertions))]
    let config = match config::AppConfig::load(&param.config_file) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    let modem_manager = match ModemManager::initialize(&config).await {
        Ok(manager) => Arc::new(manager),
        Err(err) => {
            eprintln!("Failed to initialize ModemManager: {}", err);
            std::process::exit(1);
        }
    };
    
    let sse_manager = Arc::new(api::SseManager::new());

    let webhook_manager = match config.settings.webhooks.clone() { Some(cfgs) => {
        Some(webhook::start_webhook_worker_with_concurrency(cfgs, config.settings.webhooks_max_concurrent.unwrap_or(1)))
    } _ => {
        None
    }};

    tokio::spawn(read_sms_worker(
        modem_manager.clone(),
        config.settings.read_sms_frequency,
        sse_manager.clone(),
        webhook_manager,
    ));

    if let Ok(_) = api::run_api(
        modem_manager.clone(),
        &config.settings.server_host,
        &config.settings.server_port,
        &config.settings.username.unwrap(),
        &config.settings.password.unwrap(),
        sse_manager.clone(),
    )
    .await {};
}

async fn read_sms_worker(
    modem_manager: ModemManagerRef,
    read_sms_frequency: u64,
    sse_manager: Arc<SseManager>,
    webhook_manager: Option<webhook::WebhookManager>,
) {
    loop {
        modem_manager.read_all_sms_async(
            SmsType::RecUnread,
            sse_manager.clone(),
            webhook_manager.clone(),
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(read_sms_frequency)).await;
    }
}

#[derive(Debug, StructOpt)]
pub struct Param {
#[cfg(debug_assertions)]
    #[structopt(
        short = "l",
        long = "log",
        parse(from_os_str),
        default_value = "./logs"
    )]
    pub log_path: PathBuf,

    #[cfg(not(debug_assertions))]
    #[structopt(
        short = "l",
        long = "log",
        parse(from_os_str),
        default_value = "/var/lib/sms-gateway/log"
    )]
    pub log_path: PathBuf,

    

    #[cfg(debug_assertions)]
    #[structopt(
        short = "v",
        long = "log-level",
        default_value = "debug",
        possible_values = &["off", "error", "warn", "info", "debug", "trace"]
    )]
    pub log_level: LevelFilter,

    #[cfg(not(debug_assertions))]
    #[structopt(
        short = "v",
        long = "log-level",
        default_value = "info",
        possible_values = &["off", "error", "warn", "info", "debug", "trace"]
    )]
    pub log_level: LevelFilter,

    #[structopt(
        short = "c",
        long = "config",
        parse(from_os_str),
        default_value = "/etc/sms-gateway/config.toml"
    )]
    pub config_file: PathBuf,
}

fn log_init(log_path: &PathBuf, log_level: &LevelFilter) -> anyhow::Result<()> {
    if !log_path.exists() {
        std::fs::create_dir_all(log_path)?;
    }
    let file_spec = FileSpec::default().directory(log_path);

    let _ = Logger::try_with_str(format!("{}", log_level))?
        .log_to_file(file_spec)
        .duplicate_to_stderr(Duplicate::All)
        .format_for_stderr(colored_detailed_format)
        .format_for_stdout(colored_detailed_format)
        //https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg
        .set_palette(String::from("b196;208;28;7;8"))
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        )
        .start()?;
    Ok(())
}

// SIM检测逻辑已完全移除 - 设备映射在启动时建立，运行时不再检测
