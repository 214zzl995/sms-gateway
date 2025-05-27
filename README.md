# 📱 SMS Gateway

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/214zzl995/sms-gateway)

<div align="center">
  <img src="https://raw.githubusercontent.com/214zzl995/sms-gateway/main/frontend/public/logo.svg" alt="SMS Gateway Logo" width="200">
  <h3>A modern SMS management and forwarding system</h3>
</div>

## 📖 Overview

SMS Gateway is a comprehensive solution for receiving, sending, and forwarding SMS messages through GSM modems. It provides a web-based interface for managing SMS and offers powerful webhook functionality with extensive filtering capabilities.

### Key Features

- Connect to multiple GSM modems simultaneously
- Receive and send SMS messages
- Web interface for easy management
- Forward SMS to external services via webhooks
- Advanced message filtering system
- Configurable via TOML configuration file
- PDU decoding and encoding support

## 🛠️ Building From Source

### Prerequisites

- Rust 1.60+ and Cargo
- Node.js 16+ and npm/pnpm
- SQLite

### Step 1: Build the Frontend

```bash
# Navigate to the frontend directory
cd frontend

# Install dependencies
pnpm install

# Build the frontend
pnpm run build
```

This will generate a `dist` directory within the frontend folder.

### Step 2: Build the Backend

```bash
# Return to the project root
cd ..

# Build the backend in release mode
cargo build --release
```

The compiled binary will be available in `target/release/sms-gateway`.

## 🚀 Running the Application

```bash
# Run with default settings
./target/release/sms-gateway

# Specify a custom config file
./target/release/sms-gateway --config /path/to/config.toml

# Set custom log directory
./target/release/sms-gateway --log /path/to/logs

# Set log level
./target/release/sms-gateway --log-level debug
```

## ⚙️ Configuration

The application is configured using a TOML file. By default, it looks for the config file at:
- Debug mode: `./config.toml`
- Release mode: `/etc/sms-gateway/config.toml`

### Main Configuration Sections

#### Server Settings

Settings for the web server and general application behavior in the [`[settings]`](./config.toml#L1-L6) section:

```toml
[settings]
server_host = "0.0.0.0"  # Host to bind the server to
server_port = 8951       # Port for the web interface
username = "admin"       # Web interface username
password = "password"    # Web interface password
read_sms_frequency = 60  # How often to check for new SMS (in seconds)
webhooks_max_concurrent = 10  # Maximum concurrent webhook requests
```

#### Device Configuration

Configure GSM modems in the [`[devices]`](./config.toml#L8-L10) section:

```toml
[devices.default]  # "default" is the device name
com_port = "/dev/ttyUSB1"  # Serial port path
baud_rate = 115200         # Baud rate for serial communication
```

You can add multiple devices by creating additional sections like `[devices.another]`.

#### Webhook Configuration

Configure webhooks in the [`[[settings.webhooks]]`](./config.toml#L12-L19) section:

```toml
[[settings.webhooks]]
url = "http://example.com/webhook"  # Target URL
method = "POST"                     # HTTP method
headers = { "Content-Type" = "application/json" }  # Custom headers
body = """
{
    "phone": "${contact}",
    "message": "${message}"
}
"""  # Request body template
```

### Advanced Webhook Filtering

The webhook system supports comprehensive filtering:

```toml
[[settings.webhooks]]
url = "http://example.com/webhook"
contact_filter = ["123456789"]  # Only process messages from these numbers
device_filter = ["default"]     # Only process messages from these devices
include_self_sent = false       # Exclude messages sent by the user

# Time-based filtering
time_filter = { start_time = "09:00:00", end_time = "17:00:00", days_of_week = [1, 2, 3, 4, 5] }

# Content-based filtering
message_filter = { contains = ["important"], not_contains = ["spam"], regex = "code:[0-9]{6}" }
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.
