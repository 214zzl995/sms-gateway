use std::collections::HashMap;
use chrono::NaiveDateTime;
use regex::Regex;
use crate::db::SMS;

// --------- Multipart SMS Handler ----------
struct MultipartHandler {
    // (reference number, total parts) -> (timestamp, sender, message parts, original indices)
    pending_parts: HashMap<(u8, u8), (NaiveDateTime, String, Vec<Option<String>>, Vec<u32>)>,
}

impl MultipartHandler {
    fn new() -> Self {
        Self {
            pending_parts: HashMap::new(),
        }
    }

    /// Adds a part of a multipart SMS and returns the combined message when all parts are collected
    fn add_part(
        &mut self,
        reference: u8,
        total: u8,
        current: u8,
        message: String,
        timestamp: NaiveDateTime,
        sender: String,
        index: u32,
        device: String
    ) -> Option<SMS> {
        // Validate parameters
        if current == 0 || current > total {
            return None;
        }
        
        let key = (reference, total);
        let entry = self.pending_parts
            .entry(key)
            .or_insert_with(|| (
                timestamp,
                sender.clone(),
                vec![None; total as usize],
                Vec::new(),
            ));
        
        // Store current part
        entry.2[current as usize - 1] = Some(message);
        entry.3.push(index);
        
        // Check if all parts are collected
        if entry.2.iter().all(Option::is_some) {
            let combined = entry.2.iter()
                .filter_map(|x| x.as_ref())
                .fold(String::new(), |acc, s| acc + s);
            
            Some(SMS {
                id: None,
                index: *entry.3.first().unwrap(),
                sender: Some(entry.1.clone()),
                receiver: None,
                timestamp: entry.0,
                message: combined,
                device,
                local_send: false,
            })
        } else {
            None
        }
    }
}

// ---------- Main Parser Function ----------
pub fn parse_pdu_sms(cmgl_entries: &str, device:&str) -> Vec<SMS> {
    let mut handler = MultipartHandler::new();
    let mut messages = Vec::new();
    let entry_re = Regex::new(r#"\+(CMGL): (\d+).*?\n([0-9A-F]+)"#).unwrap();
    
    for cap in entry_re.captures_iter(cmgl_entries) {
        let index = cap[2].parse().unwrap();
        let pdu = hex::decode(&cap[3]).unwrap();
        
        // Skip SMSC information
        let smsc_len = pdu[0] as usize;
        let mut pos = 1 + smsc_len;
        
        // Parse basic headers
        pos += 1; // Skip PDU type
        let sender = parse_sender(&pdu, &mut pos);
        pos += 1; // Skip protocol identifier
        let dcs = pdu[pos];
        pos += 1;
        let timestamp = parse_timestamp(&pdu[pos..pos+7]);
        pos += 7;
        
        // Parse message content
        let msg_len = pdu[pos] as usize;
        pos += 1;
        let msg_bytes = &pdu[pos..pos+msg_len];
        
        match parse_message_content(msg_bytes, dcs) {
            MessageContent::Multipart {
                reference,
                total,
                current,
                content,
            } => {
                if let Some(sms) = handler.add_part(
                    reference,
                    total,
                    current,
                    content,
                    timestamp,
                    sender.clone(),
                    index,
                    String::from(device)
                ) {
                    messages.push(sms);
                }
            }
            MessageContent::Single(content) => {
                messages.push(SMS {
                    id: None,
                    index,
                    sender: Some(sender),
                    receiver: None,
                    timestamp,
                    message: content,
                    device: device.to_string(),
                    local_send: false,
                });
            }
        }
    }
    messages
}

// ---------- Message Content Parsing ----------
enum MessageContent {
    Multipart {
        reference: u8,
        total: u8,
        current: u8,
        content: String,
    },
    Single(String),
}

fn parse_message_content(bytes: &[u8], dcs: u8) -> MessageContent {
    // Check for UDH header (indicates multipart message)
    if bytes.starts_with(&[0x05, 0x00, 0x03]) && bytes.len() > 6 {
        let reference = bytes[3];
        let total = bytes[4];
        let current = bytes[5];
        let content_bytes = &bytes[6..];
        
        MessageContent::Multipart {
            reference,
            total,
            current,
            content: decode_content(content_bytes, dcs),
        }
    } else {
        MessageContent::Single(decode_content(bytes, dcs))
    }
}

// ---------- Decoding Utilities ----------
fn parse_sender(pdu: &[u8], pos: &mut usize) -> String {
    let sender_len = pdu[*pos] as usize;
    *pos += 1;
    let sender_type = pdu[*pos];
    *pos += 1;
    let sender_bytes = &pdu[*pos..*pos + (sender_len + 1)/2];
    *pos += (sender_len + 1)/2;
    
    let mut number = decode_bcd(sender_bytes, sender_len);
    if (sender_type & 0xF0) == 0x10 {
        number.insert(0, '+');
    }
    number.trim_end_matches(|c| c == 'F' || c == '5').to_string()
}

fn decode_content(bytes: &[u8], dcs: u8) -> String {
    match dcs {
        0x08 => decode_ucs2(bytes),
        _ => bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect(),
    }
}

fn decode_ucs2(bytes: &[u8]) -> String {
    bytes.chunks_exact(2)
        .map(|ch| char::from_u32(u16::from_be_bytes([ch[0], ch[1]]) as u32).unwrap_or('�'))
        .collect()
}

fn decode_bcd(bytes: &[u8], len: usize) -> String {
    bytes.iter()
        .flat_map(|b| [b & 0x0F, (b >> 4) & 0x0F].into_iter())
        .take(len)
        .map(|n| (n + b'0') as char)
        .collect()
}

fn parse_timestamp(bytes: &[u8]) -> NaiveDateTime {
    let decode = |b| ((b & 0x0F) * 10) + (b >> 4);
    chrono::NaiveDate::from_ymd_opt(
        2000 + decode(bytes[0]) as i32,
        decode(bytes[1]) as u32,
        decode(bytes[2]) as u32,
    )
    .unwrap()
    .and_hms_opt(
        decode(bytes[3]) as u32,
        decode(bytes[4]) as u32,
        decode(bytes[5]) as u32,
    )
    .unwrap()
}
