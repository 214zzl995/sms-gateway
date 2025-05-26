use anyhow::{Context, Result};
use chrono::{NaiveTime, Weekday};
use config::{Config, File};
use fancy_regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, fmt, path::Path, str::FromStr};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub settings: Settings,
    pub devices: HashMap<String, Device>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server_host: String,
    pub server_port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub read_sms_frequency: u64,
    pub webhooks_max_concurrent: Option<usize>,
    pub webhooks: Option<Vec<WebhookConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    pub com_port: String,
    pub baud_rate: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Patch => write!(f, "PATCH"),
            Method::Head => write!(f, "HEAD"),
            Method::Options => write!(f, "OPTIONS"),
        }
    }
}

impl FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "PATCH" => Ok(Method::Patch),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            _ => Err(format!("Invalid HTTP method: {}", s)),
        }
    }
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub url: Vec<TemplateSegment>,
    pub method: Method,
    pub headers: Option<HashMap<String, Vec<TemplateSegment>>>,
    pub body: Option<Vec<TemplateSegment>>,
    pub url_params: Option<HashMap<String, Vec<TemplateSegment>>>,
    pub timeout: Option<u64>,

    // Filters
    pub contact_filter: Option<Vec<String>>, // List of contacts to include
    pub device_filter: Option<Vec<String>>,  // List of devices to include
    pub time_filter: Option<TimeFilter>,     // Time-based filtering
    pub message_filter: Option<MessageFilter>, // Content-based filtering
    pub include_self_sent: Option<bool>, // If true, include messages sent by the user in webhook
}

#[derive(Debug, Clone)]
pub struct TimeFilter {
    pub start_time: Option<NaiveTime>,      // Format: HH:MM
    pub end_time: Option<NaiveTime>,        // Format: HH:MM
    pub days_of_week: Option<Vec<Weekday>>, // 0-6, where 0 is Sunday
}

#[derive(Debug, Clone)]
pub struct MessageFilter {
    pub contains: Option<Vec<String>>, // Strings that must be in the message
    pub not_contains: Option<Vec<String>>, // Strings that must not be in the message
    pub regex: Option<Regex>,          // Regular expression pattern to match
}

#[derive(Debug, Clone)]
pub enum TemplateSegment {
    Fixed(String),
    Placeholder(Placeholder),
}

#[derive(Debug, Clone)]
pub struct Placeholder {
    pub name: SegmentName,
    pub regex: Option<Regex>,
    pub regex_name: Option<String>,
    pub regex_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum SegmentName {
    Contact,
    Timestamp,
    Message,
    Device,
    Send,
}

impl AppConfig {
    /// Load configuration from a config file
    pub fn load(config_file_path: &Path) -> Result<AppConfig> {
        // Use the `config` crate to load the config file
        let config = Config::builder()
            .add_source(File::from(config_file_path))
            .build()
            .context("Failed to load config file")?;

        // Deserialize the config file into the `AppConfig` struct
        let app_config: AppConfig = config.try_deserialize()?;

        // Validate the configuration
        test_config(&app_config)?;

        Ok(app_config)
    }
}

/// Validate required configuration fields
fn test_config(app_config: &AppConfig) -> Result<()> {
    // Validate SETTINGS section
    if app_config.settings.server_host.trim().is_empty() {
        anyhow::bail!("Fatal: server_host is not set");
    }
    if app_config.settings.server_port == 0 {
        anyhow::bail!("Fatal: server_port is not set");
    }

    // Validate DEVICES section
    for (key, device) in &app_config.devices {
        if device.com_port.trim().is_empty() {
            anyhow::bail!("Fatal: Device {} com_port is not set", key);
        }
        if device.baud_rate == 0 {
            anyhow::bail!("Fatal: Device {} baud_rate is not set", key);
        }
    }

    Ok(())
}

impl<'de> Deserialize<'de> for WebhookConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use fancy_regex::Regex;
        use serde::{de::Error, Deserialize};

        #[derive(Debug, Clone, Deserialize)]
        struct WebhookConfigDeserializer {
            pub url: String,
            pub method: String,
            pub headers: Option<HashMap<String, String>>,
            pub body: Option<String>,
            pub url_params: Option<HashMap<String, String>>,
            pub timeout: Option<u64>,
            pub contact_filter: Option<Vec<String>>,
            pub device_filter: Option<Vec<String>>,
            pub time_filter: Option<TimeFilter>,
            pub message_filter: Option<MessageFilterDeserializer>,
            pub include_self_sent: Option<bool>,
        }

        #[derive(Debug, Clone, Deserialize)]
        struct MessageFilterDeserializer {
            pub contains: Option<Vec<String>>,
            pub not_contains: Option<Vec<String>>,
            pub regex: Option<String>,
        }

        fn parse_template_segments(s: &str) -> Result<Vec<TemplateSegment>, String> {
            let mut segments = Vec::new();
            let re = match Regex::new(r"\@(.*?)\@") {
                Ok(re) => re,
                Err(e) => return Err(format!("Invalid template syntax: {}", e)),
            };

            let mut last = 0;
            for caps in re.captures_iter(s).flatten() {
                let m = caps.get(0).unwrap();
                if m.start() > last {
                    let fixed = &s[last..m.start()];
                    if !fixed.is_empty() {
                        segments.push(TemplateSegment::Fixed(fixed.to_string()));
                    }
                }
                let inner = &caps[1];
                let parts: Vec<&str> = inner.split("::").collect();
                let (name, regex, regex_name, regex_index) = match parts.len() {
                    1 => (parts[0], None, None, None),
                    2 => (parts[0], Some(parts[1].to_string()), None, None),
                    n if n >= 3 => {
                        let name = parts[0];
                        let regex = if n > 2 {
                            Some(parts[1..n - 1].join("::"))
                        } else {
                            None
                        };
                        let last = parts[n - 1];
                        if let Ok(idx) = last.parse::<usize>() {
                            (name, regex, None, Some(idx))
                        } else {
                            (name, regex, Some(last.to_string()), None)
                        }
                    }
                    _ => return Err(format!("Invalid template format in placeholder: {}", inner)),
                };

                let regex_obj = if let Some(r) = regex.as_ref() {
                    match Regex::new(r) {
                        Ok(re) => Some(re),
                        Err(e) => return Err(format!("Invalid regex in template '{}': {}", r, e)),
                    }
                } else {
                    None
                };

                let segment_name = parse_segment_name(name)?;

                segments.push(TemplateSegment::Placeholder(Placeholder {
                    name: segment_name,
                    regex: regex_obj,
                    regex_name,
                    regex_index,
                }));
                last = m.end();
            }

            if last < s.len() {
                let fixed = &s[last..];
                if !fixed.is_empty() {
                    segments.push(TemplateSegment::Fixed(fixed.to_string()));
                }
            }

            Ok(segments)
        }

        fn parse_segment_name(s: &str) -> Result<SegmentName, String> {
            match s.trim().to_lowercase().as_str() {
                "contact" => Ok(SegmentName::Contact),
                "timestamp" => Ok(SegmentName::Timestamp),
                "message" => Ok(SegmentName::Message),
                "device" => Ok(SegmentName::Device),
                "send" => Ok(SegmentName::Send),
                _ => Err(format!("Unknown segment name: '{}'. Valid names are: contact, timestamp, message, device, send", s)),
            }
        }

        fn validate_url(url_str: &str) -> Result<(), String> {
            if !(url_str.starts_with("http://") || url_str.starts_with("https://")) {
                return Err(format!(
                    "URL must start with http:// or https:// (got: {})",
                    url_str
                ));
            }

            if url_str.trim().is_empty() {
                return Err("URL cannot be empty".to_string());
            }

            Ok(())
        }

        fn validate_time_filter(filter: &Option<TimeFilter>) -> Result<(), String> {
            if let Some(tf) = filter {
                if let (Some(start), Some(end)) = (tf.start_time, tf.end_time) {
                    if start > end {
                        return Err(format!(
                            "Invalid time range: start_time {} is after end_time {}",
                            start, end
                        ));
                    }
                }
            }
            Ok(())
        }

        let raw = WebhookConfigDeserializer::deserialize(deserializer)
            .map_err(|e| D::Error::custom(format!("Failed to deserialize WebhookConfig: {}", e)))?;

        validate_url(&raw.url).map_err(|e| D::Error::custom(e))?;

        let method = Method::from_str(&raw.method).map_err(|e| D::Error::custom(e))?;

        let url = parse_template_segments(&raw.url)
            .map_err(|e| D::Error::custom(format!("Invalid URL template: {}", e)))?;

        let headers = match raw.headers {
            Some(h) => {
                let mut result = HashMap::new();
                for (k, v) in h {
                    if k.trim().is_empty() {
                        return Err(D::Error::custom("Header key cannot be empty"));
                    }
                    let segments = parse_template_segments(&v).map_err(|e| {
                        D::Error::custom(format!("Invalid header template '{}': {}", k, e))
                    })?;
                    result.insert(k, segments);
                }
                Some(result)
            }
            None => None,
        };

        let body = match raw.body {
            Some(b) => {
                let segments = parse_template_segments(&b)
                    .map_err(|e| D::Error::custom(format!("Invalid body template: {}", e)))?;
                Some(segments)
            }
            None => None,
        };

        let url_params = match raw.url_params {
            Some(h) => {
                let mut result = HashMap::new();
                for (k, v) in h {
                    if k.trim().is_empty() {
                        return Err(D::Error::custom("URL parameter key cannot be empty"));
                    }
                    let segments = parse_template_segments(&v).map_err(|e| {
                        D::Error::custom(format!("Invalid URL param template '{}': {}", k, e))
                    })?;
                    result.insert(k, segments);
                }
                Some(result)
            }
            None => None,
        };

        let message_filter = match raw.message_filter {
            Some(mf) => {
                let regex = match mf.regex {
                    Some(r) => match Regex::new(&r) {
                        Ok(re) => Some(re),
                        Err(e) => {
                            return Err(D::Error::custom(format!(
                                "Invalid regex in message filter: {}",
                                e
                            )))
                        }
                    },
                    None => None,
                };

                Some(MessageFilter {
                    contains: mf.contains,
                    not_contains: mf.not_contains,
                    regex,
                })
            }
            None => None,
        };

        validate_time_filter(&raw.time_filter).map_err(|e| D::Error::custom(e))?;

        if let Some(timeout) = raw.timeout {
            if timeout == 0 {
                return Err(D::Error::custom("Webhook timeout cannot be zero"));
            }
        }

        Ok(WebhookConfig {
            url,
            method,
            headers,
            body,
            url_params,
            timeout: raw.timeout,
            contact_filter: raw.contact_filter,
            device_filter: raw.device_filter,
            time_filter: raw.time_filter,
            message_filter,
            include_self_sent: raw.include_self_sent,
        })
    }
}

impl<'de> Deserialize<'de> for TimeFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::{de::Error, Deserialize};

        #[derive(Deserialize)]
        struct TimeFilterHelper {
            pub start_time: Option<String>,
            pub end_time: Option<String>,
            pub days_of_week: Option<Vec<serde_json::Value>>,
        }

        let helper = TimeFilterHelper::deserialize(deserializer)?;

        let start_time = match helper.start_time {
            Some(s) => Some(
                NaiveTime::parse_from_str(&s, "%H:%M")
                    .map_err(|e| D::Error::custom(format!("Invalid start_time format: {}", e)))?,
            ),
            None => None,
        };

        let end_time = match helper.end_time {
            Some(s) => Some(
                NaiveTime::parse_from_str(&s, "%H:%M")
                    .map_err(|e| D::Error::custom(format!("Invalid end_time format: {}", e)))?,
            ),
            None => None,
        };

        let days_of_week = match helper.days_of_week {
            Some(days) => {
                let mut weekdays = Vec::new();
                for day in days {
                    let weekday = match day {
                        serde_json::Value::Number(n) => {
                            let num = n
                                .as_u64()
                                .ok_or_else(|| D::Error::custom("Invalid weekday number"))?;
                            match num {
                                0 => Weekday::Sun,
                                1 => Weekday::Mon,
                                2 => Weekday::Tue,
                                3 => Weekday::Wed,
                                4 => Weekday::Thu,
                                5 => Weekday::Fri,
                                6 => Weekday::Sat,
                                _ => {
                                    return Err(D::Error::custom(format!(
                                        "Invalid weekday number: {}, must be 0-6",
                                        num
                                    )))
                                }
                            }
                        }
                        serde_json::Value::String(s) => match s.to_lowercase().as_str() {
                            "sunday" | "sun" => Weekday::Sun,
                            "monday" | "mon" => Weekday::Mon,
                            "tuesday" | "tue" => Weekday::Tue,
                            "wednesday" | "wed" => Weekday::Wed,
                            "thursday" | "thu" => Weekday::Thu,
                            "friday" | "fri" => Weekday::Fri,
                            "saturday" | "sat" => Weekday::Sat,
                            _ => {
                                return Err(D::Error::custom(format!(
                                    "Invalid weekday string: {}",
                                    s
                                )))
                            }
                        },
                        _ => {
                            return Err(D::Error::custom(
                                "Weekday must be a number (0-6) or string",
                            ))
                        }
                    };
                    weekdays.push(weekday);
                }
                Some(weekdays)
            }
            None => None,
        };

        Ok(TimeFilter {
            start_time,
            end_time,
            days_of_week,
        })
    }
}
