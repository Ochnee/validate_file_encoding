use anyhow::{anyhow, Result};
use chardetng::EncodingDetector;
use encoding_rs::*;

pub struct Detection {
    pub name: &'static str,
    pub encoding: &'static Encoding,
    pub confidence: f32,
}

pub fn detect(bytes: &[u8], expected: Option<&str>) -> Result<Detection> {
    if let Some(label) = expected {
        let encoding = Encoding::for_label(label.as_bytes())
            .ok_or_else(|| anyhow!("Unknown encoding: {}", label))?;
        return Ok(Detection {
            name: encoding.name(),
            encoding,
            confidence: 1.0,
        });
    }

    if bytes.is_empty() {
        return Ok(Detection {
            name: UTF_8.name(),
            encoding: UTF_8,
            confidence: 1.0,
        });
    }

    let mut detector = EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);

    let confidence = compute_confidence(bytes, encoding);

    Ok(Detection {
        name: encoding.name(),
        encoding,
        confidence,
    })
}

fn compute_confidence(bytes: &[u8], encoding: &'static Encoding) -> f32 {
    let has_utf8_errors = has_errors(bytes, UTF_8);
    let has_detected_errors = has_errors(bytes, encoding);

    if !has_detected_errors {
        if encoding == UTF_8 {
            return 1.0;
        }

        if !has_utf8_errors {
            let suspicious = count_suspicious_latin1_chars(bytes, UTF_8);
            if suspicious > 3 && (encoding == WINDOWS_1252) {
                return 0.85;
            }
            return 0.92;
        }
        return 0.9;
    }

    if has_utf8_errors && encoding == WINDOWS_1252 {
        return 0.95;
    }

    let error_ratio = replacement_char_count(bytes, encoding) as f32 / bytes.len().max(1) as f32;
    if error_ratio < 0.01 {
        0.8
    } else if error_ratio < 0.05 {
        0.6
    } else {
        0.3
    }
}

fn has_errors(bytes: &[u8], encoding: &'static Encoding) -> bool {
    encoding
        .decode_without_bom_handling_and_without_replacement(bytes)
        .is_none()
}

fn replacement_char_count(bytes: &[u8], encoding: &'static Encoding) -> usize {
    let (decoded, had_errors) = encoding.decode_without_bom_handling(bytes);
    if !had_errors {
        return 0;
    }
    decoded.chars().filter(|&c| c == '\u{FFFD}').count()
}

fn count_suspicious_latin1_chars(bytes: &[u8], target: &'static Encoding) -> usize {
    let (decoded, _) = target.decode_without_bom_handling(bytes);
    decoded
        .chars()
        .filter(|&c| {
            matches!(
                c,
                '\u{00C3}'
                    | '\u{00C2}'
                    | '\u{00A4}'
                    | '\u{00A5}'
                    | '\u{00A9}'
                    | '\u{00AE}'
                    | '\u{00B0}'
                    | '\u{00B1}'
                    | '\u{00B6}'
                    | '\u{00B7}'
                    | '\u{00BC}'
                    | '\u{00BD}'
                    | '\u{00BE}'
                    | '\u{00BF}'
            )
        })
        .count()
}

pub fn valid_encodings(bytes: &[u8]) -> Vec<&'static str> {
    let candidates: &[&'static Encoding] = &[
        UTF_8,
        WINDOWS_1252,
        ISO_8859_15,
        UTF_16LE,
        UTF_16BE,
    ];

    candidates
        .iter()
        .filter(|enc| is_valid_as(bytes, enc))
        .map(|enc| enc.name())
        .collect()
}

pub fn is_valid_as(bytes: &[u8], encoding: &'static Encoding) -> bool {
    encoding
        .decode_without_bom_handling_and_without_replacement(bytes)
        .is_some()
}
