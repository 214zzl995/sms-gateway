use chrono::{Local, Timelike};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{debug, error, info};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex, RwLock, Semaphore};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::api::SseManager;
use crate::config::SmsStorage;
use crate::db::{Contact, ModemSMS, SimCard, Sms};
use crate::decode::parse_pdu_sms;
use crate::webhook;

use super::pdu::{build_pdu, string_to_ucs2_pub};
use super::types::*;

const TERMINATORS: &[&[u8]] = &[
    b"\r\nOK\r\n",
    b"\r\nERROR\r\n",
    b"\r\n> ",
    b"\r\n+CME ERROR",
    b"\r\n+CMS ERROR",
];

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_millis(500);
const MAX_CONCURRENT_COMMANDS: usize = 5;

pub struct Modem {
    pub name: String,
    pub com_port: String,
    pub baud_rate: u32,
    command_tx: mpsc::UnboundedSender<ATCommand>,
    pub sim_id: RwLock<Option<String>>,
    _connection_state: Arc<RwLock<ConnectionState>>,
    _command_semaphore: Arc<Semaphore>,
    _serial_mutex: Arc<Mutex<Option<SerialStream>>>,
}

impl Modem {
    pub async fn new(com_port: &str, baud_rate: u32, name: &str) -> io::Result<Self> {
        let serial_stream = Self::create_serial_connection(com_port, baud_rate).await?;

        let (command_tx, command_rx) = mpsc::unbounded_channel::<ATCommand>();
        let serial_mutex = Arc::new(Mutex::new(Some(serial_stream)));
        let connection_state = Arc::new(RwLock::new(ConnectionState::Connected));
        let command_semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_COMMANDS));

        let name_clone = name.to_string();
        let com_port_clone = com_port.to_string();
        let serial_mutex_clone = serial_mutex.clone();
        let connection_state_clone = connection_state.clone();
        let semaphore_clone = command_semaphore.clone();

        tokio::spawn(async move {
            Self::command_processor(
                command_rx,
                serial_mutex_clone,
                &name_clone,
                &com_port_clone,
                baud_rate,
                connection_state_clone,
                semaphore_clone,
            )
            .await;
        });

        info!("device:{}, com:{} connected successfully", name, com_port);

        Ok(Modem {
            name: name.to_string(),
            com_port: com_port.to_string(),
            baud_rate,
            command_tx,
            sim_id: RwLock::new(None),
            _connection_state: connection_state,
            _command_semaphore: command_semaphore,
            _serial_mutex: serial_mutex,
        })
    }

    async fn create_serial_connection(
        com_port: &str,
        baud_rate: u32,
    ) -> tokio_serial::Result<SerialStream> {
        tokio_serial::new(com_port, baud_rate)
            .timeout(Duration::from_secs(10))
            .flow_control(tokio_serial::FlowControl::None)
            .open_native_async()
    }

    async fn command_processor(
        mut command_rx: mpsc::UnboundedReceiver<ATCommand>,
        serial_mutex: Arc<Mutex<Option<SerialStream>>>,
        name: &str,
        com_port: &str,
        baud_rate: u32,
        connection_state: Arc<RwLock<ConnectionState>>,
        semaphore: Arc<Semaphore>,
    ) {
        let mut futures = FuturesUnordered::new();

        loop {
            tokio::select! {
                Some(at_command) = command_rx.recv() => {
                    let serial_mutex = serial_mutex.clone();
                    let name = name.to_string();
                    let com_port = com_port.to_string();
                    let connection_state = connection_state.clone();
                    let semaphore = semaphore.clone();

                    futures.push(tokio::spawn(async move {
                        let _permit = semaphore.acquire().await;
                        Self::execute_command_with_retry(
                            serial_mutex,
                            at_command,
                            &name,
                            &com_port,
                            baud_rate,
                            connection_state,
                        ).await
                    }));
                }
                Some(result) = futures.next() => {
                    if let Err(e) = result {
                        error!("Command execution task failed: {}", e);
                    }
                }
                else => break,
            }
        }
    }

    async fn execute_command_with_retry(
        serial_mutex: Arc<Mutex<Option<SerialStream>>>,
        mut at_command: ATCommand,
        name: &str,
        com_port: &str,
        baud_rate: u32,
        connection_state: Arc<RwLock<ConnectionState>>,
    ) {
        while at_command.retries < MAX_RETRIES {
            {
                let state = connection_state.read().await;
                if matches!(*state, ConnectionState::Disconnected) {
                    drop(state);
                    Self::attempt_reconnection(
                        &serial_mutex,
                        com_port,
                        baud_rate,
                        &connection_state,
                        name,
                    )
                    .await;
                }
            }

            let result = {
                let mut serial_guard = serial_mutex.lock().await;
                if let Some(serial) = serial_guard.as_mut() {
                    Self::execute_single_command(serial, &at_command.command, name).await
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::NotConnected,
                        "Serial port not connected",
                    ))
                }
            };

            match result {
                Ok(response) => {
                    let _ = at_command.response_tx.send(Ok(response));
                    return;
                }
                Err(e) if e.kind() == io::ErrorKind::NotConnected => {
                    at_command.retries += 1;
                    if at_command.retries < MAX_RETRIES {
                        tokio::time::sleep(RETRY_DELAY).await;
                        continue;
                    }
                    let _ = at_command.response_tx.send(Err(e));
                    return;
                }
                Err(e) => {
                    let _ = at_command.response_tx.send(Err(e));
                    return;
                }
            }
        }

        let _ = at_command.response_tx.send(Err(io::Error::other(
            "Maximum retries exceeded",
        )));
    }

    async fn attempt_reconnection(
        serial_mutex: &Arc<Mutex<Option<SerialStream>>>,
        com_port: &str,
        baud_rate: u32,
        connection_state: &Arc<RwLock<ConnectionState>>,
        name: &str,
    ) {
        {
            let mut state = connection_state.write().await;
            if matches!(*state, ConnectionState::Reconnecting) {
                return;
            }
            *state = ConnectionState::Reconnecting;
        }

        info!("Attempting to reconnect to {} on {}", name, com_port);

        for attempt in 1..=3 {
            match Self::create_serial_connection(com_port, baud_rate).await {
                Ok(new_stream) => {
                    let mut serial_guard = serial_mutex.lock().await;
                    *serial_guard = Some(new_stream);

                    let mut state = connection_state.write().await;
                    *state = ConnectionState::Connected;

                    info!("Successfully reconnected to {} on {}", name, com_port);
                    return;
                }
                Err(e) => {
                    error!(
                        "Reconnection attempt {} failed for {}: {}",
                        attempt, name, e
                    );
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }

        let mut state = connection_state.write().await;
        *state = ConnectionState::Disconnected;
        error!("Failed to reconnect to {} after multiple attempts", name);
    }

    async fn execute_single_command(
        serial_stream: &mut SerialStream,
        command: &str,
        name: &str,
    ) -> io::Result<String> {
        debug!("TX [{}]: {}", name, Self::format_log(command));
        serial_stream.write_all(command.as_bytes()).await?;
        serial_stream.flush().await?;

        Self::read_response_buffered(serial_stream, name).await
    }

    async fn read_response_buffered(
        serial_stream: &mut SerialStream,
        name: &str,
    ) -> io::Result<String> {
        let mut buffer = Vec::with_capacity(4096);
        let mut temp_buf = [0u8; 1024];
        let timeout_duration = Duration::from_secs(30);

        let result = tokio::time::timeout(timeout_duration, async {
            loop {
                match serial_stream.read(&mut temp_buf).await {
                    Ok(0) => break,
                    Ok(bytes_read) => {
                        buffer.extend_from_slice(&temp_buf[..bytes_read]);

                        if let Some((terminator, pos)) = Self::find_terminator(&buffer) {
                            let end_pos = pos + terminator.len();
                            let response = String::from_utf8_lossy(&buffer[..end_pos]).into_owned();
                            debug!("RX [{}]: {}", name, Self::format_log(&response));
                            return Ok(response);
                        }
                    }
                    Err(e) => return Err(e),
                }
            }

            let response = String::from_utf8_lossy(&buffer).into_owned();
            debug!("RX [{}]: {}", name, Self::format_log(&response));
            Ok(response)
        })
        .await;

        match result {
            Ok(response) => response,
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("Command timeout for device {}", name),
            )),
        }
    }

    fn find_terminator(buffer: &[u8]) -> Option<(&'static [u8], usize)> {
        TERMINATORS
            .iter()
            .filter_map(|&t| {
                buffer
                    .windows(t.len())
                    .position(|w| w == t)
                    .map(|pos| (t, pos))
            })
            .max_by_key(|&(_, pos)| pos)
    }

    pub async fn init_modem(&mut self, sms_storage: Option<SmsStorage>) -> io::Result<()> {
        let init_commands = vec![
            ("ATE0\r\n", "Disable echo"),
            ("AT+CMEE=1\r\n", "Enable error messages"),
            ("AT+CMGF=0\r\n", "Set PDU mode"),
            ("AT+CSCS=\"UCS2\"\r\n", "Set character encoding"),
        ];

        for (cmd, description) in init_commands {
            if let Err(e) = self.send_command_with_ok(cmd).await {
                error!("Failed to {}: {}", description, e);
                return Err(e);
            }
        }

        if let Some(storage) = sms_storage {
            self.configure_sms_storage(storage).await?;
        }

        if let Err(e) = self.init_sim_info().await {
            log::warn!(
                "Failed to initialize SIM info for device {}: {}",
                self.name,
                e
            );
        }

        Ok(())
    }

    async fn configure_sms_storage(&self, storage: SmsStorage) -> io::Result<()> {
        let storage_str = match storage {
            SmsStorage::SIM => "SM",
            SmsStorage::ME => "ME",
            SmsStorage::MT => "MT",
        };

        let cmd = format!("AT+CPMS=\"{0}\",\"{0}\",\"{0}\"\r\n", storage_str);

        match self.send_command_with_ok(&cmd).await {
            Ok(_) => {
                info!(
                    "SMS storage set to {} for device {}",
                    storage_str, self.name
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to set SMS storage: {}", e);
                Err(e)
            }
        }
    }

    async fn init_sim_info(&mut self) -> anyhow::Result<()> {
        let (iccid_result, imsi_result, phone_result) = tokio::join!(
            self.get_sim_iccid(),
            self.get_sim_imsi(),
            self.get_phone_number()
        );

        let iccid = iccid_result.ok().flatten();
        let imsi = imsi_result.ok().flatten();
        let phone_number = phone_result.ok().flatten();

        if let Some(iccid) = iccid {
            match SimCard::find_or_create_with_phone(&iccid, imsi, phone_number).await {
                Ok(_) => {
                    *self.sim_id.write().await = Some(iccid.clone());
                    info!(
                        "SIM card initialized for device {}: ICCID={}",
                        self.name, iccid
                    );
                }
                Err(e) => {
                    error!("Failed to store SIM card info: {}", e);
                    return Err(e);
                }
            }
        } else {
            log::warn!("Could not retrieve SIM card ICCID for device {}", self.name);
        }

        Ok(())
    }

    async fn send_command_priority(&self, command: &str, priority: u8) -> io::Result<String> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        let at_command = ATCommand {
            command: command.to_string(),
            response_tx,
            _priority: priority,
            retries: 0,
        };

        self.command_tx
            .send(at_command)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Command queue closed"))?;

        response_rx
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Response channel closed"))?
    }

    async fn send_command(&self, command: &str) -> io::Result<String> {
        self.send_command_priority(command, 5).await
    }

    async fn send_command_with_ok(&self, command: &str) -> io::Result<String> {
        let response = self.send_command(command).await?;

        if response.contains("OK\r\n") {
            Ok(response)
        } else {
            error!("Command failed: {}", response);
            Err(io::Error::other(
                format!("Command failed: {}", Self::format_log(&response)),
            ))
        }
    }

    async fn send_sms_content<F>(
        &self,
        setup_cmd: &str,
        message: &str,
        transform_fn: F,
    ) -> anyhow::Result<String>
    where
        F: FnOnce(&str) -> anyhow::Result<String>,
    {
        let prompt_response = self.send_command_priority(setup_cmd, 1).await?;

        if !prompt_response.contains("> ") {
            return Err(anyhow::anyhow!("SMS prompt not received"));
        }

        let transformed_message = transform_fn(message)?;
        let full_message = format!("{}\x1A", transformed_message);

        let final_response = self.send_command_priority(&full_message, 1).await?;

        if final_response.contains("OK\r\n") && final_response.contains("+CMGS:") {
            Ok(final_response)
        } else {
            error!(
                "Incomplete SMS response: {}",
                Self::format_log(&final_response)
            );
            Err(anyhow::anyhow!("Incomplete SMS response"))
        }
    }

    pub async fn send_sms_pdu(
        &self,
        contact: &Contact,
        message: &str,
    ) -> anyhow::Result<(i64, String)> {
        info!("Sending SMS via PDU to {}: {}", contact.name, message);

        let sim_id = self.sim_id.read().await.clone().unwrap_or_default();

        let sms = Sms {
            id: 0,
            contact_id: contact.id.clone(),
            timestamp: Local::now().naive_local().with_nanosecond(0).unwrap(),
            message: message.to_string(),
            sim_id,
            send: true,
            status: crate::db::SmsStatus::Loading,
        };

        let sms_id = sms.insert().await?;

        match self.send_pdu_message(&contact.name, message).await {
            Ok(_) => {
                Sms::update_status_by_id(sms_id, crate::db::SmsStatus::Read).await?;
                Ok((sms_id, contact.id.clone()))
            }
            Err(e) => {
                Sms::update_status_by_id(sms_id, crate::db::SmsStatus::Failed).await?;
                Err(e)
            }
        }
    }

    async fn send_pdu_message(&self, phone: &str, message: &str) -> anyhow::Result<()> {
        let (pdu_data, tpdu_length) = build_pdu(phone, message)?;

        self.send_sms_content(&format!("AT+CMGS={}\r", tpdu_length), &pdu_data, |pdu| {
            Ok(pdu.to_string())
        })
        .await?;

        Ok(())
    }

    pub async fn read_sms_async_insert(
        &self,
        sms_type: SmsType,
        sse_manager: Arc<SseManager>,
        webhook_manager: Option<webhook::WebhookManager>,
    ) -> anyhow::Result<()> {
        let sms_list = self.read_sms(sms_type).await?;

        if sms_list.is_empty() {
            return Ok(());
        }

        let webhook_future = async {
            if let Some(webhook_mgr) = webhook_manager {
                for sms in &sms_list {
                    if let Err(e) = webhook_mgr.send(sms.clone()) {
                        log::error!("Failed to send webhook: {}", e);
                    }
                }
            }
        };

        let db_future = async {
            match ModemSMS::bulk_insert(&sms_list).await {
                Ok(contact_ids) => {
                    if let Ok(conversations) =
                        crate::db::Conversation::query_by_contact_ids(&contact_ids).await
                    {
                        sse_manager.send(conversations);
                    }
                }
                Err(e) => log::error!("Insert SMS error: {}", e),
            }
        };

        tokio::join!(webhook_future, db_future);
        Ok(())
    }

    pub async fn read_sms(&self, sms_type: SmsType) -> io::Result<Vec<ModemSMS>> {
        let command = format!("AT+CMGL={}\r\n", sms_type.to_at_command_pdu());
        let response = self.send_command_with_ok(&command).await?;

        let sim_id = self.sim_id.read().await.clone().unwrap_or_default();
        Ok(parse_pdu_sms(&response, &sim_id))
    }

    async fn get_modem_info<T>(
        &self,
        command: &str,
        parser: fn(&str) -> Option<T>,
    ) -> io::Result<Option<T>> {
        let raw_response = self.send_command_with_ok(command).await?;
        let cleaned_response = raw_response.trim().replace("OK", "");
        Ok(parser(&cleaned_response))
    }

    pub async fn get_signal_quality(&self) -> io::Result<Option<SignalQuality>> {
        self.get_modem_info("AT+CSQ\r\n", SignalQuality::from_response)
            .await
    }

    pub async fn check_network_registration(
        &self,
    ) -> io::Result<Option<NetworkRegistrationStatus>> {
        self.get_modem_info("AT+CREG?\r\n", NetworkRegistrationStatus::from_response)
            .await
    }

    pub async fn check_operator(&self) -> io::Result<Option<OperatorInfo>> {
        self.get_modem_info("AT+COPS?\r\n", OperatorInfo::from_response)
            .await
    }

    pub async fn get_modem_model(&self) -> io::Result<Option<ModemInfo>> {
        self.get_modem_info("AT+CGMM\r\n", ModemInfo::from_response)
            .await
    }

    pub async fn get_sim_iccid(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CCID\r\n", |response| {
            response
                .lines()
                .find(|line| {
                    line.contains("CCID:")
                        || (line.trim().len() >= 19
                            && line.trim().chars().all(|c| c.is_ascii_hexdigit()))
                })
                .map(|line| {
                    if line.contains("CCID:") {
                        line.split("CCID:")
                            .nth(1)
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|| line.trim().to_string())
                    } else {
                        line.trim().to_string()
                    }
                })
        })
        .await
    }

    pub async fn get_sim_imsi(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CIMI\r\n", |response| {
            response
                .lines()
                .find(|line| {
                    let trimmed = line.trim();
                    trimmed.len() >= 15 && trimmed.chars().all(|c| c.is_ascii_digit())
                })
                .map(|line| line.trim().to_string())
        })
        .await
    }

    pub async fn get_phone_number(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CNUM\r\n", Self::parse_phone_number)
            .await
    }

    fn parse_phone_number(response: &str) -> Option<String> {
        response
            .lines()
            .find(|line| line.starts_with("+CNUM:"))
            .and_then(|line| {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    parts[1].trim().trim_matches('"').to_string().into()
                } else {
                    None
                }
            })
    }

    fn format_log(input: &str) -> String {
        input
            .replace("\r\n", "\\r\\n")
            .replace("\r", "\\r")
            .replace("\n", "\\n")
    }

    pub async fn send_sms_text(
        &self,
        contact: &Contact,
        message: &str,
    ) -> anyhow::Result<(i64, String)> {
        info!("Sending SMS text to {}: {}", contact.name, message);

        let sim_id = self.sim_id.read().await.clone().unwrap_or_default();

        let sms = Sms {
            id: 0,
            contact_id: contact.id.clone(),
            timestamp: Local::now().naive_local().with_nanosecond(0).unwrap(),
            message: message.to_string(),
            sim_id,
            send: true,
            status: crate::db::SmsStatus::Loading,
        };

        let sms_id = sms.insert().await?;

        match self.send_text_message(&contact.name, message).await {
            Ok(_) => {
                Sms::update_status_by_id(sms_id, crate::db::SmsStatus::Read).await?;
                Ok((sms_id, contact.id.clone()))
            }
            Err(e) => {
                Sms::update_status_by_id(sms_id, crate::db::SmsStatus::Failed).await?;
                Err(e)
            }
        }
    }

    async fn send_text_message(&self, phone: &str, message: &str) -> anyhow::Result<()> {
        self.send_sms_content(&format!("AT+CMGS=\"{}\"\r", phone), message, |msg| {
            string_to_ucs2_pub(msg)
        })
        .await?;
        Ok(())
    }

    pub async fn read_sms_sync_insert(&self, sms_type: SmsType) -> anyhow::Result<()> {
        let sms_list = self.read_sms(sms_type).await?;
        if !sms_list.is_empty() {
            ModemSMS::bulk_insert(&sms_list).await?;
        }
        Ok(())
    }

    pub async fn get_sms_center(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CSCA?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CSCA:"))
                .and_then(|line| {
                    line.split('"')
                        .nth(1)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                })
        })
        .await
    }

    pub async fn get_network_info(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CPSI?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CPSI:"))
                .map(|line| line.to_string())
        })
        .await
    }

    pub async fn get_sim_status(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CPIN?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CPIN:"))
                .and_then(|line| line.split(':').nth(1).map(|s| s.trim().to_string()))
        })
        .await
    }

    pub async fn get_memory_status(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CPMS?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CPMS:"))
                .map(|line| line.to_string())
        })
        .await
    }

    pub async fn get_temperature_info(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+QTEMP?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+QTEMP:"))
                .map(|line| line.to_string())
        })
        .await
    }

    pub async fn set_sms_storage(&self, sms_storage: SmsStorage) -> io::Result<()> {
        let storage_str = match sms_storage {
            SmsStorage::SIM => "SM",
            SmsStorage::ME => "ME",
            SmsStorage::MT => "MT",
        };

        let cmd = format!("AT+CPMS=\"{0}\",\"{0}\",\"{0}\"\r\n", storage_str);

        match self.send_command_with_ok(&cmd).await {
            Ok(_) => {
                info!(
                    "SMS storage changed to {} for device {}",
                    storage_str, self.name
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to change SMS storage to {} for device {}: {}",
                    storage_str, self.name, e
                );
                Err(e)
            }
        }
    }

    pub async fn get_sms_storage_status(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CPMS?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CPMS:"))
                .map(|line| line.to_string())
        })
        .await
    }
}