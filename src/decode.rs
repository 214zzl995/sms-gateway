use chrono::NaiveDateTime;
use fancy_regex::Regex;
use std::collections::HashMap;

use crate::db::ModemSMS;

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
        sim_id: String,
    ) -> Option<ModemSMS> {
        // Validate parameters
        if current == 0 || current > total {
            return None;
        }

        let key = (reference, total);
        let entry = self.pending_parts.entry(key).or_insert_with(|| {
            (
                timestamp,
                sender.clone(),
                vec![None; total as usize],
                Vec::new(),
            )
        });

        // Store current part
        entry.2[current as usize - 1] = Some(message);
        entry.3.push(index);

        // Check if all parts are collected
        if entry.2.iter().all(Option::is_some) {
            let combined = entry
                .2
                .iter()
                .filter_map(|x| x.as_ref())
                .fold(String::new(), |acc, s| acc + s);

            log::info!("多段短信组合完成: 引用{}, 总{}段, 最终消息长度: {}", reference, total, combined.len());
            
            // Create the result before removing from pending
            let result = Some(ModemSMS {
                contact: entry.1.clone(),
                timestamp: entry.0,
                message: combined,
                send: false,
                sim_id,
            });
            
            // Remove the completed multipart message from pending
            self.pending_parts.remove(&key);
            
            result
        } else {
            let parts_received = entry.2.iter().filter(|x| x.is_some()).count();
            log::debug!("多段短信进度: 引用{}, 已收到{}/{}段", reference, parts_received, total);
            None
        }
    }
}

// ---------- Main Parser Function ----------
pub fn parse_pdu_sms(cmgl_entries: &str, sim_id: &str) -> Vec<ModemSMS> {
    let mut handler = MultipartHandler::new();
    let mut messages = Vec::new();
    let entry_re = Regex::new(r#"\+(CMGL): (\d+).*?\n([0-9A-F]+)"#).unwrap();

    let total_entries = entry_re.captures_iter(cmgl_entries).count();
    log::debug!("开始解析PDU短信: SIM ID={}, 发现{}条短信", sim_id, total_entries);

    for cap in entry_re.captures_iter(cmgl_entries).flatten() {
        let index = cap[2].parse().unwrap();
        let pdu = hex::decode(&cap[3]).unwrap();

        // Skip SMSC information
        let smsc_len = pdu[0] as usize;
        let mut pos = 1 + smsc_len;

        // Parse basic headers
        let pdu_type = pdu[pos];
        pos += 1; // Skip PDU type
        let sender = parse_sender(&pdu, &mut pos);
        pos += 1; // Skip protocol identifier
        let dcs = pdu[pos];
        pos += 1;
        let timestamp = parse_timestamp(&pdu[pos..pos + 7]);
        pos += 7;

        // Parse message content
        let udl = pdu[pos] as usize;
        pos += 1;
        
        // For SMS, take all remaining bytes as message data
        let msg_bytes = &pdu[pos..];
        
        // Check if UDH is present and adjust for GSM 7-bit decoding
        let has_udhi = (pdu_type & 0x40) != 0;

        match parse_message_content(msg_bytes, dcs, udl, has_udhi) {
            MessageContent::Multipart {
                reference,
                total,
                current,
                content,
            } => {
                log::debug!("解析到多段短信: 索引{}, 引用{}, 当前{}/{}, 内容长度: {}", 
                           index, reference, current, total, content.len());
                if let Some(sms) = handler.add_part(
                    reference,
                    total,
                    current,
                    content,
                    timestamp,
                    sender.clone(),
                    index,
                    String::from(sim_id),
                ) {
                    log::info!("多段短信完整，添加到消息列表");
                    messages.push(sms);
                }
            }
            MessageContent::Single(content) => {
                log::debug!("解析到单条短信: 索引{}, 内容长度: {}", index, content.len());
                messages.push(ModemSMS {
                    contact: sender,
                    timestamp,
                    message: content,
                    send: false,
                    sim_id: sim_id.to_string(),
                });
            }
        }
    }
    
    log::debug!("PDU解析完成: SIM ID={}, 输入{}条短信，输出{}条完整短信", sim_id, total_entries, messages.len());
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

fn parse_message_content(bytes: &[u8], dcs: u8, _udl: usize, has_udhi: bool) -> MessageContent {
    // Check for UDH header (indicates multipart message)
    if has_udhi && bytes.len() > 1 {
        let udhl = bytes[0] as usize;
        if bytes.len() > udhl + 1 {
            let udh = &bytes[1..1 + udhl];
            let content_bytes = &bytes[1 + udhl..];
            
            // Check for concatenated SMS UDH
            if udh.len() >= 5 && udh[0] == 0x00 && udh[1] == 0x03 {
                let reference = udh[2];
                let total = udh[3];
                let current = udh[4];
                
                // For GSM 7-bit, calculate the correct bit offset due to UDH
                let content = if dcs == 0x00 {
                    // For concatenated SMS with 5-byte UDH, the standard padding is 6 bits
                    let padding_bits = if udhl == 5 { 6 } else { 0 };
                    decode_gsm7bit_with_offset(content_bytes, padding_bits)
                } else {
                    decode_content(content_bytes, dcs)
                };

                return MessageContent::Multipart {
                    reference,
                    total,
                    current,
                    content,
                };
            }
        }
    }
    
    MessageContent::Single(decode_content(bytes, dcs))
}

// ---------- Decoding Utilities ----------
fn parse_sender(pdu: &[u8], pos: &mut usize) -> String {
    let sender_len = pdu[*pos] as usize;
    *pos += 1;
    let sender_type = pdu[*pos];
    *pos += 1;
    
    // For alphanumeric sender, use semi-octet representation
    // sender_len is the length of the address in semi-octets (BCD digits) 
    let sender_octets = sender_len.div_ceil(2);
    let sender_bytes = &pdu[*pos..*pos + sender_octets];
    *pos += sender_octets;
    
    match sender_type & 0x70 {
        0x50 => {
            // Alphanumeric type - decode as GSM 7-bit packed into BCD format
            decode_alphanumeric_sender(sender_bytes, sender_len)
        }
        _ => {
            // International or unknown type - use BCD decoding
            let mut number = decode_bcd(sender_bytes, sender_len);
            if (sender_type & 0x70) == 0x10 {
                number.insert(0, '+');
            }
            number.trim_end_matches(['F', 'f']).to_string()
        }
    }
}

fn decode_content(bytes: &[u8], dcs: u8) -> String {
    match dcs {
        0x00 => decode_gsm7bit(bytes),  // GSM 7-bit 默认字母表
        0x08 => decode_ucs2(bytes),     // UCS2 编码
        _ => bytes                      // 其他编码类型，保持原样
            .iter()
            .map(|b| *b as char)
            .collect()
    }
}

fn decode_alphanumeric_sender(bytes: &[u8], digit_count: usize) -> String {
    // For alphanumeric sender addresses, the digits are packed as GSM 7-bit
    // but stored in BCD format length. We need to unpack them.
    let total_bits = digit_count * 4; // Each BCD digit is 4 bits
    let mut result = String::new();
    let mut bit_buffer: u64 = 0;
    let mut bits_in_buffer = 0;
    
    // Load all bytes into bit buffer
    for &byte in bytes {
        bit_buffer |= (byte as u64) << bits_in_buffer;
        bits_in_buffer += 8;
    }
    
    // Extract 7-bit characters until we've consumed expected bits
    let mut bits_consumed = 0;
    while bits_consumed + 7 <= total_bits {
        let septet = (bit_buffer & 0x7F) as u8;
        bit_buffer >>= 7;
        bits_consumed += 7;
        
        let ch = gsm7bit_to_char(septet);
        if ch.is_ascii_graphic() && ch != '\0' {
            result.push(ch);
        } else {
            break; // Stop at first invalid character
        }
    }
    
    result
}

#[allow(dead_code)]
fn decode_gsm7bit_sender(bytes: &[u8], septets: usize) -> String {
    let mut result = String::new();
    let mut bit_buffer: u64 = 0;
    let mut bits_in_buffer = 0;
    let mut septets_decoded = 0;
    
    for &byte in bytes {
        if septets_decoded >= septets {
            break;
        }
        
        bit_buffer |= (byte as u64) << bits_in_buffer;
        bits_in_buffer += 8;
        
        while bits_in_buffer >= 7 && septets_decoded < septets {
            let septet = (bit_buffer & 0x7F) as u8;
            bit_buffer >>= 7;
            bits_in_buffer -= 7;
            septets_decoded += 1;
            
            let ch = gsm7bit_to_char(septet);
            if ch != '\0' && ch.is_ascii_graphic() {
                result.push(ch);
            }
            
            // Stop at first non-printable character or null
            if ch == '\0' || !ch.is_ascii_graphic() {
                break;
            }
        }
    }
    
    result
}

fn decode_gsm7bit_with_offset(bytes: &[u8], bit_offset: usize) -> String {
    let mut result = String::new();
    let mut bit_buffer: u64 = 0;
    let mut bits_in_buffer = bit_offset;
    
    for &byte in bytes {
        bit_buffer |= (byte as u64) << bits_in_buffer;
        bits_in_buffer += 8;
        
        while bits_in_buffer >= 7 {
            let septet = (bit_buffer & 0x7F) as u8;
            bit_buffer >>= 7;
            bits_in_buffer -= 7;
            
            let ch = gsm7bit_to_char(septet);
            if ch != '\0' {  // Skip null characters
                result.push(ch);
            }
        }
    }
    
    result.trim_end_matches('\0').to_string()
}

#[allow(dead_code)]
fn decode_gsm7bit_with_septets(bytes: &[u8], septets: usize) -> String {
    let mut result = String::new();
    let mut bit_buffer: u64 = 0;
    let mut bits_in_buffer = 0;
    let mut septets_decoded = 0;
    
    for &byte in bytes {
        if septets_decoded >= septets {
            break;
        }
        
        bit_buffer |= (byte as u64) << bits_in_buffer;
        bits_in_buffer += 8;
        
        while bits_in_buffer >= 7 && septets_decoded < septets {
            let septet = (bit_buffer & 0x7F) as u8;
            bit_buffer >>= 7;
            bits_in_buffer -= 7;
            septets_decoded += 1;
            
            let ch = gsm7bit_to_char(septet);
            result.push(ch);
        }
    }
    
    result
}

fn gsm7bit_to_char(septet: u8) -> char {
    match septet {
                0x00 => '@',
                0x01 => '£',
                0x02 => '$',
                0x03 => '¥',
                0x04 => 'è',
                0x05 => 'é',
                0x06 => 'ù',
                0x07 => 'ì',
                0x08 => 'ò',
                0x09 => 'Ç',
                0x0A => '\n',
                0x0B => 'Ø',
                0x0C => 'ø',
                0x0D => '\r',
                0x0E => 'Å',
                0x0F => 'å',
                0x10 => '\u{0394}', // Δ
                0x11 => '_',
                0x12 => '\u{03A6}', // Φ
                0x13 => '\u{0393}', // Γ
                0x14 => '\u{039B}', // Λ
                0x15 => '\u{03A9}', // Ω
                0x16 => '\u{03A0}', // Π
                0x17 => '\u{03A8}', // Ψ
                0x18 => '\u{03A3}', // Σ
                0x19 => '\u{0398}', // Θ
                0x1A => '\u{039E}', // Ξ
                0x1B => '\u{001B}', // escape for extended table
                0x1C => 'Æ',
                0x1D => 'æ',
                0x1E => 'ß',
                0x1F => 'É',
                0x20 => ' ',
                0x21..=0x7F => septet as char,
                _ => '?'
    }
}

fn decode_gsm7bit(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut bit_buffer: u16 = 0;
    let mut bits_in_buffer = 0;

    for &byte in bytes {
        bit_buffer |= (byte as u16) << bits_in_buffer;
        bits_in_buffer += 8;

        while bits_in_buffer >= 7 {
            let septet = (bit_buffer & 0x7F) as u8;
            bit_buffer >>= 7;
            bits_in_buffer -= 7;
            
            result.push(gsm7bit_to_char(septet));
        }
    }

    result.trim_end_matches('\0').to_string()
}

fn decode_ucs2(bytes: &[u8]) -> String {
    bytes
        .chunks_exact(2)
        .map(|ch| char::from_u32(u16::from_be_bytes([ch[0], ch[1]]) as u32).unwrap_or('�'))
        .collect()
}

fn decode_bcd(bytes: &[u8], len: usize) -> String {
    bytes
        .iter()
        .flat_map(|b| [b & 0x0F, (b >> 4) & 0x0F].into_iter())
        .take(len)
        .map(|n| (n + b'0') as char)
        .collect()
}

fn parse_timestamp(bytes: &[u8]) -> NaiveDateTime {
    let decode = |b| ((b & 0x0F) * 10) + (b >> 4);
    
    let year = 2000 + decode(bytes[0]) as i32;
    let month = decode(bytes[1]) as u32;
    let day = decode(bytes[2]) as u32;
    let hour = decode(bytes[3]) as u32;
    let minute = decode(bytes[4]) as u32;
    let second = decode(bytes[5]) as u32;
    
    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|date| date.and_hms_opt(hour, minute, second))
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap())
}
