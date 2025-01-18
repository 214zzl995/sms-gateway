use log::{error, info};
use serialport::SerialPort;
use std::io::{self, Read, Write};
use std::time::Duration;

/// GSM Modem
struct GSMModem {
    com_port: String,
    baud_rate: u32,
    port: Option<Box<dyn SerialPort>>,
    device_id: String,
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
                Err(_) => String::new(), // 忽略错误
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

    /// Sned sms
    fn send_sms(&mut self, mobile: &str, message: &str) -> String {
        info!("--- SendSMS {}: {}", mobile, message);

        self.send(&format!("AT+CMGS=\"{}\"\r", mobile)); // 应该返回 ">"
        self.read(3);

        // EOM CTRL-Z = 26
        self.send_command(&format!("{}\x1A", message), true)
    }

    /// Log escaping
    fn transpose_log(&self, input: &str) -> String {
        input.replace("\r\n", "\\r\\n").replace("\r", "\\r")
    }
}
