use std::{collections::HashMap, path::PathBuf, sync::Arc};

use db::{db_init, SMS};
use flexi_logger::{
    colored_detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming,
    WriteMode,
};
use log::{error, LevelFilter};
use modem::SmsType;
use structopt::StructOpt;

mod config;
mod db;
mod modem;

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

    let mut modems = HashMap::new();

    for (name, device) in &config.devices {
        let modem = match modem::Modem::new(&device.com_port, device.baud_rate, name) {
            Ok(mut modem) => {
                if let Err(err) = modem.init_modem().await {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
                modem
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        };
        modems.insert(name.clone(), modem);
    }
    let modes_arc = Arc::new(modems);

    tokio::spawn(read_sms_worker(
        modes_arc.clone(),
        config.settings.read_sms_frequency,
    ));

    loop {
    }
}

async fn read_sms_worker(devices: Arc<HashMap<String, modem::Modem>>, read_sms_frequency: u64) {
    let modem_keys = devices.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
    loop {
        for key in &modem_keys {
            let modem = devices.get(key).unwrap();
            match modem.read_sms(SmsType::All).await {
                Ok(smss) => {
                    for sms in &smss {
                        log::info!("SMS: {:?}", sms);
                    }

                    if let Err(err) = SMS::bulk_insert(&smss).await{
                        error!("{}",err);
                    }
                }
                Err(err) => {
                    log::error!("Read SMS error: {}", err);
                    continue;
                }
            };
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(read_sms_frequency)).await;
    }
}

#[derive(Debug, StructOpt)]
pub struct Param {
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
        std::fs::create_dir_all(&log_path)?;
    }
    let file_spec = FileSpec::default().directory(log_path);

    let _ = Logger::try_with_str(format!("{}", log_level.to_string()))?
        .write_mode(WriteMode::BufferAndFlush)
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
