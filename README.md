# SMS Gateway - Multi-device SMS Gateway System
 
*Web-based Multi-device SMS Management Platform*

## ✨ Features

- **Multi-device Management**  
    - Connect multiple GSM devices simultaneously.
    - Display device status in real-time (signal strength, carrier, network status).

## 🚀 Quick Start

### Binary Installation (Linux/macOS)


## ⚙️ Configuration Instructions

Edit `config.toml`:

```toml
[settings]
server_host = "0.0.0.0"
server_port = 8951
username = "admin"
password = "password778"
read_sms_frequency = 60

[devices.default]
com_port = "/dev/ttyUSB2"
baud_rate = 115200

[devices.test]
com_port = "/dev/ttyUSB4"
baud_rate = 115200
```