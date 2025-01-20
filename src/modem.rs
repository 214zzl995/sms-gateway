use log::{error, info};
use serialport::SerialPort;
use std::io::{self, Read, Write};
use std::time::Duration;

/// Enum representing the type of SMS messages
#[derive(Debug, Clone, Copy)]
enum SmsType {
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
struct SMS {
    index: u32,        // 短信索引
    status: String,    // 短信状态（如 "REC READ"）
    sender: String,    // 发送者号码
    timestamp: String, // 时间戳（如 "25/01/15,15:19:52+32"）
    message: String,   // 短信内容
}

/// GSM Modem
struct GSMModem {
    com_port: String,
    baud_rate: u32,
    port: Option<Box<dyn SerialPort>>,
    device_id: String,
}

impl SMS {
    fn decode_message(&mut self) {
        //use ucs2
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

impl GSMModem {
    /// Create a new instance of GSMModem
    fn new(com_port: String, baud_rate: u32, device_id: String) -> Self {
        GSMModem {
            com_port,
            baud_rate,
            port: None,
            device_id,
        }
    }

    /// Connect to the serial port device
    fn connect(&mut self) -> io::Result<()> {
        let builder = serialport::new(&self.com_port, self.baud_rate);

        let port = builder.timeout(Duration::from_secs(1)).open()?;
        self.port = Some(port);
        self.init_modem();
        Ok(())
    }

    /// Initialize the modem
    fn init_modem(&mut self) {
        self.send_command("ATE0\r\n", true); // echo off
        self.send_command("AT+CMEE=1\r\n", true); // useful error messages
        self.send_command("AT+WIND=0\r\n", true); // disable notifications
        self.send_command("AT+CMGF=1\r\n", true); // switch to TEXT mode
    }

    /// Send the command and wait for the possible response
    fn send_command(&mut self, command: &str, wait_for_ok: bool) -> String {
        self.send(command);

        if wait_for_ok {
            let possibilities = vec!["OK\r\n", "ERROR\r\n"];
            match self.expect(&possibilities) {
                Ok(output) => output,
                Err(_) => String::new(), // ignore errors
            }
        } else {
            self.read(1)
        }
    }

    /// Send data to the serial port
    fn send(&mut self, command: &str) {
        info!("--- Send: {}", self.transpose_log(command));
        if let Some(port) = &mut self.port {
            port.write_all(command.as_bytes()).unwrap();
        }
    }

    /// Read data from the serial port
    fn read(&mut self, n: usize) -> String {
        let mut output = String::new();
        let mut buf = vec![0; n];

        if let Some(port) = &mut self.port {
            let _ = port.read(&mut buf);
            output = String::from_utf8_lossy(&buf).to_string();
        }

        info!("--- Read({}): {}", n, self.transpose_log(&output));
        output
    }

    /// Wait for a specific response
    fn expect(&mut self, possibilities: &[&str]) -> io::Result<String> {
        let read_max = possibilities.iter().map(|s| s.len()).max().unwrap_or(0) + 2;
        let mut buf = vec![0; read_max];
        let mut status = String::new();

        if let Some(port) = &mut self.port {
            for _ in 0..read_max {
                let n = port.read(&mut buf)?;
                if n > 0 {
                    status = String::from_utf8_lossy(&buf[..n]).to_string();

                    for possibility in possibilities {
                        if status.ends_with(possibility) {
                            info!(
                                "--- Expect: {} Got: {}",
                                self.transpose_log(&possibilities.join("|")),
                                self.transpose_log(&status)
                            );
                            return Ok(status);
                        }
                    }
                }
            }
        }

        error!(
            "--- Expect: {} Got: {} (match not found!)",
            self.transpose_log(&possibilities.join("|")),
            self.transpose_log(&status)
        );
        Err(io::Error::new(io::ErrorKind::Other, "match not found"))
    }

    /// Send SMS
    fn send_sms(&mut self, mobile: &str, message: &str) -> String {
        info!("--- SendSMS {}: {}", mobile, message);

        self.send(&format!("AT+CMGS=\"{}\"\r", mobile)); // should return ">"
        self.read(3);

        // EOM CTRL-Z = 26
        self.send_command(&format!("{}\x1A", message), true)
    }

    /// Read SMS messages based on the specified type
    fn read_sms(&mut self, sms_type: SmsType) -> io::Result<Vec<SMS>> {
        // Send the AT command to list SMS messages
        let command = format!("AT+CMGL=\"{}\"\r\n", sms_type.to_at_command());
        self.send(&command);

        // Read the response
        let mut response = String::new();
        let mut buf = [0; 128];
        loop {
            if let Some(port) = &mut self.port {
                let n = port.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                response.push_str(&String::from_utf8_lossy(&buf[..n]));
                if response.contains("OK\r\n") || response.contains("ERROR\r\n") {
                    break;
                }
            }
        }

        // Parse the response into SMS structs
        let sms_list = parse_sms_response(&response);
        Ok(sms_list)
    }

    /// Log escaping
    fn transpose_log(&self, input: &str) -> String {
        input.replace("\r\n", "\\r\\n").replace("\r", "\\r")
    }
}

/// Parse the response from AT+CMGL command into a list of SMS structs
fn parse_sms_response(response: &str) -> Vec<SMS> {
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
                    });
                    i += 1; // Skip the message line
                }
            }
        }
        i += 1;
    }

    sms_list
}

#[test]
fn parse_sms_response_test() {
    let smss = r#"
+CMGL: 0,"REC READ","106917966088001",,"25/01/15,15:19:52+32"
3010004100700070006C00653011004100700070006C006500208D2662374EE378014E3AFF1A00370032003600330039003130028BF752FF4E0E4ED64EBA51714EAB3002
+CMGL: 1,"REC READ","106917966088001",,"25/01/15,15:55:16+32"
3010004100700070006C00653011004100700070006C006500208D2662374EE378014E3AFF1A00350031003500340038003130028BF752FF4E0E4ED64EBA51714EAB3002
+CMGL: 2,"REC READ","19106857179",,"25/01/17,23:05:49+32"
6D4B8BD54E004E0B
    "#;
    let sms_list = parse_sms_response(smss);
    for mut sms in sms_list {
        sms.decode_message();
        println!("{:?}", sms);
    }
}
