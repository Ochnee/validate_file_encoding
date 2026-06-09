use crate::detect;
use encoding_rs::*;

pub struct Warning {
    pub offset: usize,
    pub length: usize,
    pub warning_type: String,
    pub message: String,
    pub bytes: Vec<u8>,
    pub suggested_char: Option<String>,
    pub line: usize,
}

pub fn scan(bytes: &[u8], detected: &crate::detect::Detection) -> Vec<Warning> {
    let mut warnings = Vec::new();
    let encoding = detected.encoding;

    let line_map = build_byte_line_map(bytes);

    check_replacement_chars(bytes, encoding, &line_map, &mut warnings);
    check_utf8_as_latin1(bytes, encoding, &line_map, &mut warnings);
    check_latin1_as_utf8(bytes, encoding, &line_map, &mut warnings);

    warnings
}

fn build_byte_line_map(bytes: &[u8]) -> Vec<usize> {
    let mut map = Vec::new();
    map.push(0);
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            map.push(i + 1);
        }
    }
    map
}

fn byte_offset_to_line(byte_offset: usize, line_map: &[usize]) -> usize {
    for (line_num, &start) in line_map.iter().enumerate().rev() {
        if byte_offset >= start {
            return line_num + 1;
        }
    }
    1
}

fn check_replacement_chars(
    bytes: &[u8],
    encoding: &'static Encoding,
    _line_map: &[usize],
    warnings: &mut Vec<Warning>,
) {
    let (decoded, had_errors) = encoding.decode_without_bom_handling(bytes);
    if !had_errors {
        return;
    }

    let count = decoded.chars().filter(|&c| c == '\u{FFFD}').count();
    if count > 0 {
        warnings.push(Warning {
            offset: 0,
            length: 0,
            warning_type: "replacement_char".into(),
            message: format!(
                "{} replacement character(s) U+FFFD — decoding errors in this file",
                count
            ),
            bytes: vec![],
            suggested_char: None,
            line: 1,
        });
    }
}

fn check_utf8_as_latin1(
    bytes: &[u8],
    encoding: &'static Encoding,
    line_map: &[usize],
    warnings: &mut Vec<Warning>,
) {
    let name = encoding.name();
    if name == "UTF-8" {
        return;
    }

    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == 0xC3 && (0x80..=0xBF).contains(&bytes[i + 1]) {
            let byte2 = bytes[i + 1] as u16;
            let codepoint = ((bytes[i] as u16 & 0x1F) << 6) | (byte2 & 0x3F);

            let line = byte_offset_to_line(i, line_map);

            let suggested = std::char::from_u32(codepoint as u32).map(|c| c.to_string());

            let latin1_chars = format!("{}{}", bytes[i] as char, bytes[i + 1] as char);

            let msg = format!(
                "Bytes 0x{:02X} 0x{:02X} look like UTF-8→Latin-1 mojibake — \"{}\" \
                 read as \"{}\"",
                bytes[i],
                bytes[i + 1],
                suggested.as_deref().unwrap_or("?"),
                latin1_chars,
            );

            warnings.push(Warning {
                offset: i,
                length: 2,
                warning_type: "utf8_as_latin1".into(),
                message: msg,
                bytes: vec![bytes[i], bytes[i + 1]],
                suggested_char: suggested,
                line,
            });

            i += 2;
        } else {
            i += 1;
        }
    }
}

fn check_latin1_as_utf8(
    bytes: &[u8],
    encoding: &'static Encoding,
    line_map: &[usize],
    warnings: &mut Vec<Warning>,
) {
    let name = encoding.name();
    if name != "UTF-8" {
        return;
    }

    if detect::is_valid_as(bytes, WINDOWS_1252) {
        let latin1_specific = bytes.iter().any(|&b| {
            matches!(
                b,
                0xE5 | 0xE4 | 0xF6 | 0xC5 | 0xC4 | 0xD6 | 0xE6 | 0xF8 | 0xC6 | 0xD8
                    | 0xE9 | 0xE8 | 0xFC | 0xF1 | 0xE0 | 0xF5
            )
        });

        let utf8_errors = has_utf8_errors(bytes);

        if utf8_errors || latin1_specific {
            let byte_positions = find_latin1_bytes(bytes);

            for &(offset, byte_val) in &byte_positions {
                let line = byte_offset_to_line(offset, line_map);
                let expected_char = (byte_val as char).to_string();
                warnings.push(Warning {
                    offset,
                    length: 1,
                    warning_type: "latin1_as_utf8".into(),
                    message: format!(
                        "Byte 0x{:02X} is invalid UTF-8 but valid Latin-1 \"{}\" \
                         — possible Latin-1→UTF-8 mojibake",
                        byte_val, expected_char,
                    ),
                    bytes: vec![byte_val],
                    suggested_char: Some(expected_char),
                    line,
                });
            }

            if byte_positions.len() > 2 {
                let line = byte_offset_to_line(byte_positions[0].0, line_map);
                warnings.push(Warning {
                    offset: byte_positions[0].0,
                    length: byte_positions.len(),
                    warning_type: "latin1_as_utf8".into(),
                    message: format!(
                        "{} bytes are valid Latin-1 but invalid UTF-8 — file may actually \
                         be Latin-1 encoded",
                        byte_positions.len(),
                    ),
                    bytes: vec![],
                    suggested_char: None,
                    line,
                });
            }
        }
    }
}

fn has_utf8_errors(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_err()
}

fn find_latin1_bytes(bytes: &[u8]) -> Vec<(usize, u8)> {
    let mut positions = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b < 0x80 {
            i += 1;
            continue;
        }

        if is_valid_utf8_start(b) {
            let seq_len = utf8_sequence_length(b);
            if i + seq_len <= bytes.len() && is_valid_utf8_sequence(bytes, i, seq_len) {
                i += seq_len;
                continue;
            }
        }

        if b & 0x80 != 0 {
            positions.push((i, b));
        }
        i += 1;
    }
    positions
}

fn is_valid_utf8_start(b: u8) -> bool {
    (0xC2..=0xF4).contains(&b)
}

fn utf8_sequence_length(b: u8) -> usize {
    if b & 0xE0 == 0xC0 {
        2
    } else if b & 0xF0 == 0xE0 {
        3
    } else if b & 0xF8 == 0xF0 {
        4
    } else {
        1
    }
}

fn is_valid_utf8_sequence(bytes: &[u8], start: usize, len: usize) -> bool {
    for j in 1..len {
        if start + j >= bytes.len() || bytes[start + j] & 0xC0 != 0x80 {
            return false;
        }
    }

    match len {
        2 => {
            let b1 = bytes[start];
            !(b1 == 0xC0 || b1 == 0xC1)
        }
        3 => {
            let b1 = bytes[start];
            let b2 = bytes[start + 1];
            !(b1 == 0xE0 && b2 < 0xA0) && !(b1 == 0xED && b2 > 0x9F)
        }
        4 => {
            let b1 = bytes[start];
            let b2 = bytes[start + 1];
            !(b1 == 0xF0 && b2 < 0x90) && !(b1 == 0xF4 && b2 > 0x8F)
        }
        _ => false,
    }
}

pub fn looks_binary(bytes: &[u8]) -> bool {
    let sample = if bytes.len() > 4096 {
        &bytes[..4096]
    } else {
        bytes
    };

    if sample.is_empty() {
        return false;
    }

    let null_count = sample.iter().filter(|&&b| b == 0x00).count();
    if null_count > 0 && null_count as f64 / sample.len() as f64 > 0.05 {
        return true;
    }

    let text_count = sample
        .iter()
        .filter(|&&b| (0x20..=0x7E).contains(&b) || b == b'\t' || b == b'\n' || b == b'\r')
        .count();

    if (text_count as f64 / sample.len() as f64) < 0.50 {
        return true;
    }

    let control_count = sample
        .iter()
        .filter(|&&b| {
            b == 0x00 || b == 0x7F || b < 0x08 || (b > 0x0D && b < 0x20)
        })
        .count();

    if (control_count as f64 / sample.len() as f64) > 0.10 {
        return true;
    }

    false
}
