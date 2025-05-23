use chrono::{Local, Timelike};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serialport::SerialPort;
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::api::SseManager;
use crate::db::{Contact, ModemSMS, SMS};
use crate::decode::parse_pdu_sms;
use crate::webhook;

const TERMINATORS: &[&[u8]] = &[
    b"\r\nOK\r\n",
    b"\r\nERROR\r\n",
    b"\r\n> ",
    b"\r\n+CME ERROR",
    b"\r\n+CMS ERROR",
];

/// Enum representing the type of SMS messages
#[derive(Debug, Clone, Copy)]
pub enum SmsType {
    RecUnread,
    RecRead,
    StoUnsent,
    StoSent,
    All,
}

impl SmsType {
    fn to_at_command_pdu(&self) -> u8 {
        match self {
            SmsType::RecUnread => 0,
            SmsType::RecRead => 1,
            SmsType::StoUnsent => 2,
            SmsType::StoSent => 3,
            SmsType::All => 4,
        }
    }
}

/// GSM Modem
pub struct Modem {
    pub name: String,
    pub com_port: String,
    pub baud_rate: u32,
    port: Mutex<Box<dyn SerialPort + Send>>,
}

impl Modem {
    /// Create a new instance of GSMModem
    pub fn new(com_port: &str, baud_rate: u32, name: &str) -> io::Result<Self> {
        let port = serialport::new(com_port, baud_rate)
            .timeout(Duration::from_secs(60))
            .open()?;

        info!("device:{},com:{} connected successfully", name, com_port);

        Ok(Modem {
            name: name.to_string(),
            com_port: com_port.to_string(),
            baud_rate,
            port: Mutex::new(port),
        })
    }

    /// Initialize the modem
    pub async fn init_modem(&mut self) -> io::Result<()> {
        self.send_command_with_ok("ATE0\r\n").await?; // echo off
        self.send_command_with_ok("AT+CMEE=1\r\n").await?; // useful error messages
        self.send_command_with_ok("AT+CMGF=0\r\n").await?; // switch to TEXT mode
        self.send_command_with_ok("AT+CSCS=\"UCS2\"\r\n").await?; // Set the character set to UCS2.
        Ok(())
    }

    /// Send command and expect "OK" response
    async fn send_command_with_ok(&self, command: &str) -> io::Result<String> {
        let mut port = self.port.lock().await;
        self.send_locked(command, &mut port)?;
        let response = self.read_to_string_locked(&mut port)?;

        if response.contains("OK\r\n") {
            Ok(response)
        } else {
            error!("Command failed: {}", response);
            Err(io::Error::new(io::ErrorKind::Other, "Missing OK response"))
        }
    }

    /// Wait for SMS prompt and send message content
    async fn send_sms_content<F>(
        &self,
        setup_cmd: &str,
        message: &str,
        transform_fn: F,
    ) -> anyhow::Result<String>
    where
        F: FnOnce(&str) -> anyhow::Result<String>,
    {
        // Phase 1: Initialize SMS sending process
        let mut port = self.port.lock().await;
        self.send_locked(setup_cmd, &mut port)?;

        // Wait for prompt
        let mut prompt_response = String::new();
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < Duration::from_secs(5) {
            let mut buffer = [0u8; 1];
            if port.read(&mut buffer).is_ok() {
                prompt_response.push(buffer[0] as char);
                if prompt_response.ends_with("> ") {
                    break;
                }
            }
        }

        if !prompt_response.contains("> ") {
            return Err(anyhow::anyhow!(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "SMS prompt not received",
            )));
        }

        // Phase 2: Send message content with EOM (CTRL-Z)
        let full_message = format!("{}\x1A", transform_fn(message)?);
        self.send_locked(&full_message, &mut port)?;

        // Phase 3: Handle multi-line response
        let mut final_response = String::new();
        let mut ok_received = false;
        let mut cmgs_received = false;
        let timeout = Duration::from_secs(10);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout {
            let mut buffer = [0u8; 128];
            match port.read(&mut buffer) {
                Ok(bytes_read) => {
                    let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
                    final_response.push_str(&chunk);

                    cmgs_received = cmgs_received || final_response.contains("+CMGS:");
                    ok_received = ok_received || final_response.contains("OK\r\n");

                    if ok_received && cmgs_received {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
                Err(e) => return Err(anyhow::anyhow!(e)),
            }
        }

        drop(port);
        if ok_received && cmgs_received {
            Ok(final_response)
        } else {
            error!(
                "Incomplete SMS response: {}",
                self.transpose_log(&final_response)
            );
            Err(anyhow::anyhow!(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Incomplete SMS response: {}", final_response),
            )))
        }
    }

    /// Internal helper function to prepare SMS and handle common send logic
    async fn prepare_and_send_sms<F, Fut>(
        &self,
        contact: &Contact,
        message: &str,
        log_prefix: &str,
        send_fn: F,
    ) -> anyhow::Result<(i64, String)>
    where
        F: FnOnce(i64) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<()>>,
    {
        info!("{} to {}: {}", log_prefix, contact.name, message);

        let contact_id = contact.id.clone();

        // Create SMS record
        let sms = SMS {
            id: 0,
            contact_id: contact_id.clone(),
            timestamp: Local::now().naive_local().with_nanosecond(0).unwrap(),
            message: message.to_string(),
            device: self.name.clone(),
            send: true,
            status: crate::db::SmsStatus::Loading,
        };

        // Insert SMS record
        let sms_id = match sms.insert().await {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to insert SMS: {}", err);
                return Err(anyhow::anyhow!(err));
            }
        };

        // Call the provided send function
        match send_fn(sms_id).await {
            Ok(()) => {
                if let Err(err) = SMS::update_status_by_id(sms_id, crate::db::SmsStatus::Read).await
                {
                    error!("Failed to update SMS status to Read: {}", err);
                }
                Ok((sms_id, contact_id))
            }
            Err(err) => {
                if let Err(update_err) =
                    SMS::update_status_by_id(sms_id, crate::db::SmsStatus::Failed).await
                {
                    error!("Failed to update SMS status to Failed: {}", update_err);
                }
                error!("Failed to send SMS: {}", err);
                Err(err)
            }
        }
    }

    /// Send SMS message with enhanced response handling
    pub async fn send_sms_text(
        &self,
        contact: &Contact,
        message: &str,
    ) -> anyhow::Result<(i64, String)> {
        self.prepare_and_send_sms(contact, message, "Sending SMS", |_| async {
            self.send_sms_content(&format!("AT+CMGS=\"{}\"\r", contact.name), message, |msg| {
                Ok(string_to_ucs2(msg)?)
            })
            .await?;
            Ok(())
        })
        .await
    }

    /// Send SMS message in PDU mode (GSM 03.38/03.40 standard)
    pub async fn send_sms_pdu(
        &self,
        contact: &Contact,
        message: &str,
    ) -> anyhow::Result<(i64, String)> {
        self.prepare_and_send_sms(contact, message, "Sending SMS via PDU", |_| async {
            // PDU encoding
            let (pdu_data, tpdu_length) = build_pdu(&contact.name, message)?;

            self.send_sms_content(&format!("AT+CMGS={}\r", tpdu_length), &pdu_data, |pdu| {
                Ok(pdu.to_string())
            })
            .await?;

            Ok(())
        })
        .await
    }
    pub async fn read_sms_async_insert(
        &self,
        sms_type: SmsType,
        sse_manager: Arc<SseManager>,
        webhook_manager: Option<webhook::WebhookManager>,
    ) -> anyhow::Result<()> {
        let sms_list = self.read_sms(sms_type).await?;
        if !sms_list.is_empty() {
            let sms_list_clone = sms_list.clone();
            if let Some(webhook_mgr) = webhook_manager.clone() {
                tokio::spawn(async move {
                    for sms in sms_list_clone {
                        if let Err(e) = webhook_mgr.send(sms) {
                            log::error!("Failed to send webhook: {}", e);
                        }
                    }
                });
            }

            tokio::spawn(async move {
                if let Ok(contact_ids) = ModemSMS::bulk_insert(&sms_list).await {
                    tokio::spawn(async move {
                        if let Ok(conversations) =
                            crate::db::Conversation::query_by_contact_ids(&contact_ids).await
                        {
                            sse_manager.send(conversations);
                        }
                    });
                } else {
                    log::error!("Insert SMS error");
                }
            });
        }
        Ok(())
    }

    pub async fn read_sms_sync_insert(&self, sms_type: SmsType) -> anyhow::Result<()> {
        let sms_list = self.read_sms(sms_type).await?;
        if !sms_list.is_empty() {
            ModemSMS::bulk_insert(&sms_list).await?;
        }
        Ok(())
    }

    /// Read SMS messages based on the specified type
    pub async fn read_sms(&self, sms_type: SmsType) -> io::Result<Vec<ModemSMS>> {
        let command = format!("AT+CMGL=\"{}\"\r\n", sms_type.to_at_command_pdu());
        let response = self.send_command_with_ok(&command).await?;
        debug!("ReadSMS: {}", response);
        Ok(parse_pdu_sms(&response, &self.name))
    }

    /// Get modem status info with simple pattern
    async fn get_modem_info<T>(
        &self,
        command: &str,
        parser: fn(&str) -> Option<T>,
    ) -> io::Result<Option<T>> {
        let response = self
            .send_command_with_ok(command)
            .await?
            .trim()
            .to_string()
            .replace("OK", "");
        Ok(parser(&response))
    }

    /// Get signal strength (RSSI) and Bit Error Rate (BER)
    pub async fn get_signal_quality(&self) -> io::Result<Option<SignalQuality>> {
        self.get_modem_info("AT+CSQ\r\n", SignalQuality::from_response)
            .await
    }

    /// Check network registration status
    pub async fn check_network_registration(
        &self,
    ) -> io::Result<Option<NetworkRegistrationStatus>> {
        self.get_modem_info("AT+CREG?\r\n", NetworkRegistrationStatus::from_response)
            .await
    }

    /// Check current operator
    pub async fn check_operator(&self) -> io::Result<Option<OperatorInfo>> {
        self.get_modem_info("AT+COPS?\r\n", OperatorInfo::from_response)
            .await
    }

    /// Get modem model
    pub async fn get_modem_model(&self) -> io::Result<Option<ModemInfo>> {
        self.get_modem_info("AT+CGMM\r\n", ModemInfo::from_response)
            .await
    }

    /// Log escaping
    fn transpose_log(&self, input: &str) -> String {
        input.replace("\r\n", "\\r\\n").replace("\r", "\\r")
    }

    /// Internal send method (requires held lock)
    fn send_locked(&self, command: &str, port: &mut Box<dyn SerialPort + Send>) -> io::Result<()> {
        debug!("TX [{}]: {}", self.name, self.transpose_log(command));
        port.write_all(command.as_bytes())?;
        port.flush()?;
        Ok(())
    }

    /// Internal read method (requires held lock)
    fn read_to_string_locked(&self, port: &mut Box<dyn SerialPort + Send>) -> io::Result<String> {
        let mut buffer = Vec::new();
        let mut temp_buf = [0u8; 1024];

        loop {
            match port.read(&mut temp_buf) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }

                    buffer.extend_from_slice(&temp_buf[..bytes_read]);

                    if let Some((matched_term, pos)) = TERMINATORS
                        .iter()
                        .filter_map(|t| {
                            buffer
                                .windows(t.len())
                                .position(|w| w == *t)
                                .map(|pos| (t, pos))
                        })
                        .max_by_key(|&(_, pos)| pos)
                    {
                        let end_pos = pos + matched_term.len();
                        let response = String::from_utf8_lossy(&buffer[..end_pos]).to_string();
                        buffer.drain(..end_pos);
                        debug!("Found terminator: {:?}", matched_term);
                        return Ok(response);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    if !buffer.is_empty() {
                        let response = String::from_utf8_lossy(&buffer).to_string();
                        buffer.clear();
                        debug!("Returning partial response after timeout");
                        return Ok(response);
                    }
                    break;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        let output = String::from_utf8_lossy(&buffer).to_string();
        buffer.clear();
        debug!("RX [{}]: {}", self.name, self.transpose_log(&output));
        Ok(output)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalQuality {
    rssi: i32,
    ber: i32,
}

impl SignalQuality {
    pub fn from_response(response: &str) -> Option<Self> {
        if let Some(data) = response.split(":").nth(1) {
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(rssi), Ok(ber)) = (
                    parts[0].trim().parse::<i32>(),
                    parts[1].trim().parse::<i32>(),
                ) {
                    return Some(SignalQuality { rssi, ber });
                }
            }
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkRegistrationStatus {
    status: String,
    location_area_code: Option<String>,
    cell_id: Option<String>,
}

impl NetworkRegistrationStatus {
    pub fn from_response(response: &str) -> Option<Self> {
        if let Some(data) = response.split(":").nth(1) {
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() >= 2 {
                let status = parts[0].trim().to_string();
                let location_area_code = parts.get(1).map(|s| s.trim().to_string());
                let cell_id = parts.get(2).map(|s| s.trim().to_string());
                return Some(NetworkRegistrationStatus {
                    status,
                    location_area_code,
                    cell_id,
                });
            }
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperatorInfo {
    operator_name: String,
    operator_id: String,
    registration_status: String,
}

impl OperatorInfo {
    pub fn from_response(response: &str) -> Option<Self> {
        if let Some(data) = response.split(":").nth(1) {
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() >= 3 {
                let registration_status = parts[0].trim().to_string();
                let operator_name = parts[2].trim_matches('"').to_string();
                let operator_id = parts[1].trim().to_string();
                return Some(OperatorInfo {
                    operator_name,
                    operator_id,
                    registration_status,
                });
            }
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModemInfo {
    model: String,
}

impl ModemInfo {
    pub fn from_response(response: &str) -> Option<Self> {
        Some(ModemInfo {
            model: response.trim().to_string(),
        })
    }
}

/// Encode a text message to UCS2 hex string
fn string_to_ucs2(message: &str) -> anyhow::Result<String> {
    let encoded: Vec<u16> = message.encode_utf16().collect();

    if encoded.len() > 70 {
        return Err(anyhow::anyhow!(
            "UCS2 message too long (max 70 characters), current: {} characters",
            encoded.len()
        ));
    }

    let mut bytes = Vec::with_capacity(encoded.len() * 2);
    for code_unit in encoded {
        bytes.extend_from_slice(&code_unit.to_be_bytes());
    }

    Ok(hex::encode_upper(&bytes))
}

/// Parse a phone number and format for PDU
fn parse_number(number: &str) -> anyhow::Result<(u8, String)> {
    let cleaned_number = number.trim_start_matches('+').replace(" ", "");
    let addr_type = if number.starts_with('+') { 0x91 } else { 0x81 };
    let mut swapped = String::new();
    let mut chars: Vec<char> = cleaned_number.chars().collect();

    if chars.len() % 2 != 0 {
        chars.push('F');
    }

    for i in 0..chars.len() / 2 {
        let pos = i * 2;
        swapped.push(chars[pos + 1]);
        swapped.push(chars[pos]);
    }

    Ok((addr_type, swapped))
}

/// Encode the complete TPDU for a message
fn encode_tpdu(mobile: &str, message: &str) -> anyhow::Result<(String, usize)> {
    let first_octet = "11"; // SMS-SUBMIT, no reply path, no status report, no validity period
    let mr = "00";

    let destination_phone = mobile.trim_start_matches('+').replace(" ", "");
    let phone_len = format!("{:02X}", destination_phone.len());
    let (addr_type, swapped_number) = parse_number(mobile)?;
    let destination = format!("{}{:02X}{}", phone_len, addr_type, swapped_number);

    let pid = "00";
    let dcs = "08";
    let encoded_text = string_to_ucs2(message)?;
    let udl = format!("{:02X}", message.chars().count() * 2);
    let vp = "00";

    let tpdu = format!(
        "{}{}{}{}{}{}{}{}",
        first_octet, mr, destination, pid, dcs, vp, udl, encoded_text
    );

    let tpdu_length = tpdu.len() / 2;

    Ok((tpdu, tpdu_length))
}

/// Build the complete PDU for sending
fn build_pdu(mobile: &str, message: &str) -> anyhow::Result<(String, usize)> {
    let smsc_info = "00"; // Length 0 = use default SMSC
    let (tpdu, tpdu_length) = encode_tpdu(mobile, message)?;
    let full_pdu = format!("{}{}", smsc_info, tpdu);

    Ok((full_pdu, tpdu_length))
}
