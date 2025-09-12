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

    Ok(hex::encode_upper(bytes))
}

fn parse_number(number: &str) -> anyhow::Result<(u8, String)> {
    let cleaned_number = number.trim_start_matches('+').replace(' ', "");
    let addr_type = if number.starts_with('+') { 0x91 } else { 0x81 };

    let mut chars: Vec<char> = cleaned_number.chars().collect();
    if chars.len() % 2 != 0 {
        chars.push('F');
    }

    let mut swapped = String::with_capacity(chars.len());
    for chunk in chars.chunks(2) {
        if chunk.len() == 2 {
            swapped.push(chunk[1]);
            swapped.push(chunk[0]);
        }
    }

    Ok((addr_type, swapped))
}

fn encode_tpdu(mobile: &str, message: &str) -> anyhow::Result<(String, usize)> {
    const FIRST_OCTET: &str = "11";
    const MESSAGE_REF: &str = "00";
    const PID: &str = "00";
    const DCS: &str = "08";
    const VP: &str = "00";

    let destination_phone = mobile.trim_start_matches('+').replace(' ', "");
    let phone_len = format!("{:02X}", destination_phone.len());
    let (addr_type, swapped_number) = parse_number(mobile)?;
    let destination = format!("{}{:02X}{}", phone_len, addr_type, swapped_number);

    let encoded_text = string_to_ucs2(message)?;
    let udl = format!("{:02X}", message.chars().count() * 2);

    let tpdu = format!(
        "{}{}{}{}{}{}{}{}",
        FIRST_OCTET, MESSAGE_REF, destination, PID, DCS, VP, udl, encoded_text
    );

    let tpdu_length = tpdu.len() / 2;
    Ok((tpdu, tpdu_length))
}

pub fn build_pdu(mobile: &str, message: &str) -> anyhow::Result<(String, usize)> {
    const SMSC_INFO: &str = "00";
    let (tpdu, tpdu_length) = encode_tpdu(mobile, message)?;
    let full_pdu = format!("{}{}", SMSC_INFO, tpdu);
    Ok((full_pdu, tpdu_length))
}

pub fn string_to_ucs2_pub(message: &str) -> anyhow::Result<String> {
    string_to_ucs2(message)
}