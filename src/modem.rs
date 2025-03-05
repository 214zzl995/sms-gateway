use chrono::{Local, Timelike};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serialport::SerialPort;
use std::io::{self, Read, Write};
use std::time::Duration;
use tokio::sync::Mutex;

use crate::db::SMS;
use crate::decode::parse_pdu_sms;

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
    /// Convert the enum variant to its corresponding AT command string
    fn _to_at_command_text(&self) -> &'static str {
        match self {
            SmsType::RecUnread => "REC UNREAD",
            SmsType::RecRead => "REC READ",
            SmsType::StoUnsent => "STO UNSENT",
            SmsType::StoSent => "STO SENT",
            SmsType::All => "ALL",
        }
    }

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
        let builder = serialport::new(com_port, baud_rate);

        let port = builder.timeout(Duration::from_secs(10)).open()?;
        info!("device:{},com:{} connected successfully", name, com_port);

        let modem = Modem {
            name: name.to_string(),
            com_port: com_port.to_string(),
            baud_rate,
            port: Mutex::new(port),
        };

        Ok(modem)
    }

    /// Initialize the modem
    pub async fn init_modem(&mut self) -> io::Result<()> {
        self.send_command_with_ok("ATE0\r\n").await?; // echo off
        self.send_command_with_ok("AT+CMEE=1\r\n").await?; // useful error messages
        self.send_command_with_ok("AT+CMGF=0\r\n").await?; // switch to TEXT mode

        Ok(())
    }

    /// Send command and expect "OK" response (maintains continuous lock)
    async fn send_command_with_ok(&self, command: &str) -> io::Result<String> {
        // Acquire lock at the start and maintain through entire operation
        let mut port = self.port.lock().await;

        // Combined atomic send-receive operation
        self.send_locked(command, &mut port)?;
        let response = self.read_to_string_locked(&mut port)?;

        if response.contains("OK\r\n") {
            Ok(response)
        } else {
            error!("Command failed: {}", response);
            Err(io::Error::new(io::ErrorKind::Other, "Missing OK response"))
        }
    }

    /// Send command without checking OK response (maintains continuous lock)
    async fn _send_command_without_ok(&self, command: &str) -> io::Result<String> {
        let mut port = self.port.lock().await;

        self.send_locked(command, &mut port)?;
        self.read_to_string_locked(&mut port)
    }
    /// Send data to the serial port
    async fn _send(&self, command: &str) -> io::Result<()> {
        debug!("Device:{} Send: {}", self.name, self.transpose_log(command));
        let port = &mut self.port.lock().await;
        let _ = port.write_all(command.as_bytes())?;
        port.flush()?;
        Ok(())
    }

    /// Read data from the serial port into a string
    async fn _read_to_string(&self) -> io::Result<String> {
        let mut buffer = [0u8; 1024];
        let port = &mut self.port.lock().await;
        let bytes_read = port.read(&mut buffer)?;
        let output = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        debug!("Device:{} Read: {}", self.name, self.transpose_log(&output));
        Ok(output)
    }

    /// Send SMS message with enhanced response handling
    pub async fn send_sms(&self, mobile: &str, message: &str) -> io::Result<String> {
        info!("Sending SMS to {}: {}", mobile, message);

        // Phase 1: Initialize SMS sending process
        let mut port = self.port.lock().await;
        self.send_locked(&format!("AT+CMGS=\"{}\"\r", mobile), &mut port)?;

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

        // Validate prompt reception
        if !prompt_response.contains("> ") {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "SMS prompt not received",
            ));
        }

        // Phase 2: Send message content with EOM (CTRL-Z)
        let full_message = format!("{}\x1A", message);
        self.send_locked(&full_message, &mut port)?;

        // Phase 3: Handle multi-line response
        let mut final_response = String::new();
        let mut ok_received = false;
        let mut cmgs_received = false;
        let timeout = Duration::from_secs(10);
        let start_time = std::time::Instant::now();

        // Read response chunks until timeout
        while start_time.elapsed() < timeout {
            let mut buffer = [0u8; 128];
            match port.read(&mut buffer) {
                Ok(bytes_read) => {
                    // Accumulate response chunks
                    let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
                    final_response.push_str(&chunk);

                    // Check for required response markers
                    cmgs_received = cmgs_received || final_response.contains("+CMGS:");
                    ok_received = ok_received || final_response.contains("OK\r\n");

                    // Early exit when both markers found
                    if ok_received && cmgs_received {
                        break;
                    }
                }
                // Handle non-fatal timeouts
                Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
                Err(e) => return Err(e),
            }
        }

        // Final response validation
        if ok_received && cmgs_received {
            let sms = SMS {
                index: 0,
                id: None,
                sender: None,
                receiver: Some(mobile.to_string()),
                timestamp: Local::now().naive_local().with_nanosecond(0).unwrap(),
                message: message.to_string(),
                device: self.name.clone(),
                local_send: true,
            };
            tokio::spawn(async move {
                let _ = sms.insert().await.is_err_and(|err| {
                    error!("{}", err);
                    true
                });
            });

            Ok(final_response)
        } else {
            error!(
                "Incomplete SMS response: {}",
                self.transpose_log(&final_response)
            );
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Incomplete SMS response: {}", final_response),
            ))
        }
    }

    pub async fn read_sms_async_insert(&self, sms_type: SmsType) -> anyhow::Result<()> {
        let sms_list = self.read_sms(sms_type).await?;
        tokio::spawn(async move {
            if let Err(err) = SMS::bulk_insert(&sms_list).await {
                log::error!("Insert SMS error: {}", err);
            };
        });
        Ok(())
    }

    pub async fn read_sms_sync_insert(&self, sms_type: SmsType) -> anyhow::Result<()> {
        let sms_list = self.read_sms(sms_type).await?;
        SMS::bulk_insert(&sms_list).await?;
        Ok(())
    }

    /// Read SMS messages based on the specified type
    pub async fn read_sms(&self, sms_type: SmsType) -> io::Result<Vec<SMS>> {
        // Send the AT command to list SMS messages
        let command = format!("AT+CMGL=\"{}\"\r\n", sms_type.to_at_command_pdu());

        // Read the response
        let response = self.send_command_with_ok(&command).await?;
        debug!("ReadSMS: {}", response);

        // Parse the response into SMS structs
        let sms_list = parse_pdu_sms(&response, &self.name);

        Ok(sms_list)
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
                        break; // Handle EOF
                    }

                    // Append new data to buffer
                    buffer.extend_from_slice(&temp_buf[..bytes_read]);

                    // Scan for termination patterns in real-time
                    if let Some((matched_term, pos)) = TERMINATORS
                        .iter()
                        .filter_map(|t| {
                            buffer
                                .windows(t.len())
                                .position(|w| w == *t)
                                .map(|pos| (t, pos)) // Return both terminator and position
                        })
                        .max_by_key(|&(_, pos)| pos)
                    // Find last occurring terminator
                    {
                        let end_pos = pos + matched_term.len();
                        let response = String::from_utf8_lossy(&buffer[..end_pos]).to_string();
                        buffer.drain(..end_pos); // Efficiently remove processed data
                        debug!("Found terminator: {:?}", matched_term);
                        return Ok(response);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    // Check for partial data before timeout handling
                    if !buffer.is_empty() {
                        let response = String::from_utf8_lossy(&buffer).to_string();
                        buffer.clear();
                        debug!("Returning partial response after timeout");
                        return Ok(response);
                    }
                    break;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Non-blocking I/O handling
                    break;
                }
                Err(e) => {
                    // Propagate other errors
                    return Err(e);
                }
            }
        }

        // Process remaining data in buffer
        let output = String::from_utf8_lossy(&buffer).to_string();
        buffer.clear(); // Explicitly clear buffer
        debug!("RX [{}]: {}", self.name, self.transpose_log(&output));
        Ok(output)
    }

    fn _read_to_string_locked(&self, port: &mut Box<dyn SerialPort + Send>) -> io::Result<String> {
        let mut buffer = [0u8; 1024];
        let bytes_read = port.read(&mut buffer)?;
        let output = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        debug!("RX [{}]: {}", self.name, self.transpose_log(&output));
        Ok(output)
    }

    /// Get signal strength (RSSI) and Bit Error Rate (BER)
    pub async fn get_signal_quality(&self) -> io::Result<Option<SignalQuality>> {
        let response = self
            .send_command_with_ok("AT+CSQ\r\n")
            .await?
            .trim()
            .to_string()
            .replace("OK", "");
        Ok(SignalQuality::from_response(&response))
    }

    /// Check network registration status
    pub async fn check_network_registration(
        &self,
    ) -> io::Result<Option<NetworkRegistrationStatus>> {
        let response = self
            .send_command_with_ok("AT+CREG?\r\n")
            .await?
            .trim()
            .to_string()
            .replace("OK", "");
        Ok(NetworkRegistrationStatus::from_response(&response))
    }

    /// Check current operator
    pub async fn check_operator(&self) -> io::Result<Option<OperatorInfo>> {
        let response = self
            .send_command_with_ok("AT+COPS?\r\n")
            .await?
            .trim()
            .to_string()
            .replace("OK", "")
            .to_string();
        debug!("Current Operator: {}", response);
        Ok(OperatorInfo::from_response(&response))
    }

    /// Get modem model
    pub async fn get_modem_model(&self) -> io::Result<Option<ModemInfo>> {
        let response = self
            .send_command_with_ok("AT+CGMM\r\n")
            .await?
            .trim()
            .to_string()
            .replace("OK", "")
            .to_string();
        debug!("Modem Model: {}", response);
        Ok(ModemInfo::from_response(&response))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalQuality {
    rssi: i32, // Signal Strength (RSSI)
    ber: i32,  // Bit Error Rate (BER)
}

impl SignalQuality {
    // Parse AT+CSQ response (e.g., "+CSQ: 19,0")
    pub fn from_response(response: &str) -> Option<Self> {
        // Extract the part after "+CSQ:"
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
    status: String, // Registration status ("0" = Not registered, "1" = Registered, etc.)
    location_area_code: Option<String>,
    cell_id: Option<String>,
}

impl NetworkRegistrationStatus {
    // Parse AT+CREG? response (e.g., "+CREG: 0,1")
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
    // Parse AT+COPS? response (e.g., "+COPS: 0,0,\"Vodafone\",2")
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
    // Parse AT+CGMM response (e.g., "Model ABC123")
    pub fn from_response(response: &str) -> Option<Self> {
        Some(ModemInfo {
            model: response.trim().to_string(),
        })
    }
}
