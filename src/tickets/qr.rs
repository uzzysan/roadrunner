use qrcode::QrCode;
use qrcode::render::svg::Color;
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;
use crate::errors::{AppError, AppResult};

/// Generuje kod QR jako SVG base64
/// 
/// # Arguments
/// * `data` - Dane do zakodowania
/// 
/// # Returns
/// * `String` - QR code jako base64 SVG
pub fn generate_qr_code(data: &str) -> AppResult<String> {
    let qr = QrCode::new(data)
        .map_err(|e| AppError::Internal(format!("Failed to generate QR: {}", e)))?;

    let svg = qr.render::<Color>()
        .min_dimensions(300, 300)
        .dark_color(Color("#000000"))
        .light_color(Color("#FFFFFF"))
        .build();

    let base64_svg = general_purpose::STANDARD.encode(svg);

    Ok(format!("data:image/svg+xml;base64,{}", base64_svg))
}

/// Generuje unikalny kod biletu
/// 
/// Format: TICKET:<uuid>:<timestamp>
/// 
/// # Returns
/// * `String` - Unikalny kod biletu
pub fn generate_ticket_code() -> String {
    let uuid = Uuid::new_v4();
    let timestamp = chrono::Utc::now().timestamp();
    format!("TICKET:{}:{}", uuid, timestamp)
}

/// Generuje QR code dla biletu
/// 
/// # Returns
/// * `(String, String)` - (kod biletu, QR code base64)
pub fn generate_ticket_qr() -> AppResult<(String, String)> {
    let ticket_code = generate_ticket_code();
    let qr_code = generate_qr_code(&ticket_code)?;
    Ok((ticket_code, qr_code))
}

/// Weryfikuje format kodu biletu
/// 
/// # Arguments
/// * `code` - Kod do weryfikacji
/// 
/// # Returns
/// * `bool` - true jeśli format jest poprawny
pub fn is_valid_ticket_code(code: &str) -> bool {
    // Format: TICKET:<uuid>:<timestamp>
    let parts: Vec<&str> = code.split(':').collect();
    if parts.len() != 3 {
        return false;
    }

    if parts[0] != "TICKET" {
        return false;
    }

    // Sprawdź czy druga część to valid UUID
    if Uuid::parse_str(parts[1]).is_err() {
        return false;
    }

    // Sprawdź czy trzecia część to valid timestamp
    if parts[2].parse::<i64>().is_err() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ticket_code() {
        let code = generate_ticket_code();
        assert!(code.starts_with("TICKET:"));
        assert!(is_valid_ticket_code(&code));
    }

    #[test]
    fn test_generate_qr_code() {
        let qr = generate_qr_code("test data").unwrap();
        assert!(qr.starts_with("data:image/svg+xml;base64,"));
    }

    #[test]
    fn test_generate_ticket_qr() {
        let (code, qr) = generate_ticket_qr().unwrap();
        assert!(is_valid_ticket_code(&code));
        assert!(qr.starts_with("data:image/svg+xml;base64,"));
    }

    #[test]
    fn test_is_valid_ticket_code() {
        assert!(is_valid_ticket_code("TICKET:550e8400-e29b-41d4-a716-446655440000:1234567890"));
        assert!(!is_valid_ticket_code("INVALID:code"));
        assert!(!is_valid_ticket_code("TICKET:invalid-uuid:123"));
        assert!(!is_valid_ticket_code("TICKET:550e8400-e29b-41d4-a716-446655440000:invalid"));
    }
}
