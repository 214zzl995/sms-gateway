use log::{error, info};
use serialport::SerialPort;
use std::io::{self, Read, Write};
use std::time::Duration;
use tokio::sync::Mutex;

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
    fn to_at_command(&self) -> &'static str {
        match self {
            SmsType::RecUnread => "REC UNREAD",
            SmsType::RecRead => "REC READ",
            SmsType::StoUnsent => "STO UNSENT",
            SmsType::StoSent => "STO SENT",
            SmsType::All => "ALL",
        }
    }
}

/// SMS struct representing a single SMS message
#[derive(Debug)]
pub struct SMS {
    index: u32,        // SMS index
    status: String,    // SMS status (e.g., "REC READ")
    sender: String,    // Sender's phone number
    timestamp: String, // Timestamp (e.g., "25/01/15,15:19:52+32")
    message: String,   // SMS content
    device: String,
}

/// GSM Modem
pub struct Modem {
    name: String,
    com_port: String,
    baud_rate: u32,
    port: Mutex<Box<dyn SerialPort + Send>>,
}

impl SMS {
    fn decode_message(&mut self) {
        let mut decoded = String::new();
        let mut chars = self.message.chars().collect::<Vec<_>>();

        // Process the encoded string in chunks of 4 characters
        while chars.len() >= 4 {
            // Take 4 characters as a UCS2 code point
            let chunk: String = chars.drain(0..4).collect();
            let code_point = u32::from_str_radix(&chunk, 16).unwrap_or(0);

            // Convert the code point to a Unicode character
            if let Some(c) = char::from_u32(code_point) {
                decoded.push(c);
            } else {
                decoded.push('�'); // Replacement character for invalid code points
            }
        }
        self.message = decoded;
    }
}

impl Modem {
    /// Create a new instance of GSMModem
    pub fn new(com_port: &str, baud_rate: u32, name: &str) -> io::Result<Self> {
        let builder = serialport::new(com_port, baud_rate);

        let port = builder.timeout(Duration::from_secs(1)).open()?;
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
        self.send_command_with_ok("AT+WIND=0\r\n").await?; // disable notifications
        self.send_command_with_ok("AT+CMGF=1\r\n").await?; // switch to TEXT mode

        Ok(())
    }

    /// Send a command and expect an "OK" response.
    /// If the response does not contain "OK", return an error.
    async fn send_command_with_ok(&self, command: &str) -> io::Result<String> {
        self.send(command);
        let response = self.read_to_string().await?;

        // Check if the response contains "OK"
        if response.contains("OK\r\n") {
            Ok(response)
        } else {
            error!(
                "--- Expected OK, but got: {}",
                self.transpose_log(&response)
            );
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Expected OK, but got: {}", response),
            ))
        }
    }

    /// Send a command without expecting an "OK" response.
    /// Simply return the response as a string.
    async fn send_command_without_ok(&mut self, command: &str) -> io::Result<String> {
        self.send(command).await?;
        self.read_to_string().await
    }

    /// Send data to the serial port
    async fn send(&self, command: &str) -> io::Result<()> {
        info!("--- Send: {}", self.transpose_log(command));
        let port = &mut self.port.lock().await;
        let _ = port.write_all(command.as_bytes())?;
        port.flush()?;
        Ok(())
    }

    /// Read data from the serial port into a string
    async fn read_to_string(&self) -> io::Result<String> {
        let mut output = String::new();
        let _ = &mut self.port.lock().await.read_to_string(&mut output)?;
        info!("--- Read: {}", self.transpose_log(&output));
        Ok(output)
    }

    /// Send SMS
    async fn send_sms(&mut self, mobile: &str, message: &str) -> io::Result<String> {
        info!("--- SendSMS {}: {}", mobile, message);

        self.send_command_without_ok(&format!("AT+CMGS=\"{}\"\r", mobile))
            .await?; // should return ">"

        // EOM CTRL-Z = 26
        self.send_command_with_ok(&format!("{}\x1A", message)).await
    }

    /// Read SMS messages based on the specified type
    pub async fn read_sms(&self, sms_type: SmsType) -> io::Result<Vec<SMS>> {
        // Send the AT command to list SMS messages
        let command = format!("AT+CMGL=\"{}\"\r\n", sms_type.to_at_command());

        // Read the response
        let response = self.send_command_with_ok(&command).await?;
        info!("--- ReadSMS: {}", response);

        // Parse the response into SMS structs
        let mut sms_list = parse_sms_response(&response, &self.com_port);
        sms_list.iter_mut().for_each(|sms| sms.decode_message());
        Ok(sms_list)
    }

    /// Log escaping
    fn transpose_log(&self, input: &str) -> String {
        input.replace("\r\n", "\\r\\n").replace("\r", "\\r")
    }
}
/// Parse the response from AT+CMGL command into a list of SMS structs
fn parse_sms_response(response: &str, device: &str) -> Vec<SMS> {
    let mut sms_list = Vec::new();
    let lines: Vec<&str> = response.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("+CMGL:") {
            // Parse the header line
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let index = parts[0]
                    .split(':')
                    .nth(1)
                    .unwrap_or("0")
                    .trim()
                    .parse::<u32>()
                    .unwrap_or(0);

                let status = parts[1].trim_matches('"').to_string();
                let sender = parts[2].trim_matches('"').to_string();
                let timestamp =
                    parts[4].trim_matches('"').to_string() + " " + parts[5].trim_matches('"');

                // Parse the message content (next line)
                if i + 1 < lines.len() {
                    let message = lines[i + 1].trim().to_string();
                    sms_list.push(SMS {
                        index,
                        status,
                        sender,
                        timestamp,
                        message,
                        device: device.to_string(),
                    });
                    i += 1; // Skip the message line
                }
            }
        }
        i += 1;
    }

    sms_list
}
