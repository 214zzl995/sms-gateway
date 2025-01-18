use std::path::PathBuf;

use flexi_logger::{
    colored_detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming,
    WriteMode,
};
use structopt::StructOpt;

mod config;
mod modem;

#[tokio::main]
async fn main() {
    let param = Param::from_args();
    log_init(&param.log_path).unwrap();
    let config = config::AppConfig::load(&param.config_file).unwrap();
    
}

#[derive(Debug, StructOpt)]
pub struct Param {
    #[structopt(
        short = "l",
        long = "log",
        parse(from_os_str),
        default_value = "/var/log/sms-gateway"
    )]
    pub log_path: PathBuf,

    #[structopt(
        short = "c",
        long = "config",
        parse(from_os_str),
        default_value = "/etc/sms-gateway/config.toml"
    )]
    pub config_file: PathBuf,
}

fn log_init(log_path: &PathBuf) -> anyhow::Result<()> {
    if !log_path.exists() {
        std::fs::create_dir_all(&log_path)?;
    }
    let file_spec = FileSpec::default().directory(log_path);

    let _ = Logger::try_with_str("info,pago_mqtt=error,paho_mqtt_c=error")?
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
