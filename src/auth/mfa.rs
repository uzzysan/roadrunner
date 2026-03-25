use crate::errors::{AppError, AppResult};
use base64::{engine::general_purpose, Engine as _};
use totp_rs::{Algorithm, QrCode, Secret, TOTP};

/// Konfiguracja TOTP
const TOTP_ISSUER: &str = "RoadRunner";
const TOTP_ACCOUNT_NAME: &str = "RoadRunner User";

/// Generuje nowy sekret TOTP dla użytkownika
///
/// # Returns
/// * `(String, String)` - (sekret, URL dla QR code)
pub fn generate_totp_secret(user_email: &str) -> AppResult<(String, String)> {
    let secret = Secret::generate_secret()
        .to_bytes()
        .map_err(|e| AppError::Internal(format!("Failed to generate secret: {}", e)))?;

    let secret_base32 = general_purpose::STANDARD.encode(&secret);

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.clone())
        .map_err(|e| AppError::Internal(format!("Failed to create TOTP: {}", e)))?;

    let qr_url = totp.get_uri(TOTP_ISSUER.to_string(), user_email.to_string());

    Ok((secret_base32, qr_url))
}

/// Weryfikuje kod TOTP
///
/// # Arguments
/// * `secret` - Sekret TOTP (base64)
/// * `code` - Kod do weryfikacji
///
/// # Returns
/// * `bool` - true jeśli kod jest prawidłowy
pub fn verify_totp(secret: &str, code: &str) -> AppResult<bool> {
    let secret_bytes = general_purpose::STANDARD
        .decode(secret)
        .map_err(|e| AppError::Internal(format!("Invalid secret: {}", e)))?;

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)
        .map_err(|e| AppError::Internal(format!("Failed to create TOTP: {}", e)))?;

    let is_valid = totp
        .check_current(code)
        .map_err(|e| AppError::Internal(format!("TOTP verification failed: {}", e)))?;

    Ok(is_valid)
}

/// Generuje QR code jako base64 PNG
///
/// # Arguments
/// * `qr_url` - URL do zakodowania w QR
///
/// # Returns
/// * `String` - QR code jako base64 PNG
pub fn generate_qr_code(qr_url: &str) -> AppResult<String> {
    let qr = QrCode::new(qr_url)
        .map_err(|e| AppError::Internal(format!("Failed to generate QR: {}", e)))?;

    let image = qr
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .build();

    let base64_qr = general_purpose::STANDARD.encode(image);

    Ok(format!("data:image/svg+xml;base64,{}", base64_qr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_totp() {
        let (secret, _qr_url) = generate_totp_secret("test@example.com").unwrap();

        // Generuj aktualny kod
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            general_purpose::STANDARD.decode(&secret).unwrap(),
        )
        .unwrap();

        let code = totp.generate_current().unwrap();

        // Weryfikuj kod
        let is_valid = verify_totp(&secret, &code).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_invalid_code() {
        let (secret, _qr_url) = generate_totp_secret("test@example.com").unwrap();

        let is_valid = verify_totp(&secret, "000000").unwrap();
        assert!(!is_valid);
    }
}
