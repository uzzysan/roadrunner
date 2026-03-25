pub mod stripe;

use crate::models::payment::PaymentStatus;

/// Sprawdza czy płatność została zakończona sukcesem
pub fn is_payment_successful(status: PaymentStatus) -> bool {
    matches!(status, PaymentStatus::Succeeded)
}

/// Sprawdza czy płatność jest w trakcie
pub fn is_payment_pending(status: PaymentStatus) -> bool {
    matches!(status, PaymentStatus::Pending)
}

/// Sprawdza czy płatność może być anulowana
pub fn can_cancel_payment(status: PaymentStatus) -> bool {
    matches!(status, PaymentStatus::Pending)
}

/// Formatuje kwotę do wyświetlenia (cents -> decimal)
pub fn format_amount(amount: i32, currency: &str) -> String {
    let decimal = amount as f64 / 100.0;
    match currency {
        "PLN" => format!("{:.2} zł", decimal),
        "EUR" => format!("€{:.2}", decimal),
        "USD" => format!("${:.2}", decimal),
        _ => format!("{:.2} {}", decimal, currency),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::payment::PaymentStatus;

    #[test]
    fn test_is_payment_successful() {
        assert!(is_payment_successful(PaymentStatus::Succeeded));
        assert!(!is_payment_successful(PaymentStatus::Pending));
        assert!(!is_payment_successful(PaymentStatus::Failed));
    }

    #[test]
    fn test_is_payment_pending() {
        assert!(is_payment_pending(PaymentStatus::Pending));
        assert!(!is_payment_pending(PaymentStatus::Succeeded));
    }

    #[test]
    fn test_can_cancel_payment() {
        assert!(can_cancel_payment(PaymentStatus::Pending));
        assert!(!can_cancel_payment(PaymentStatus::Succeeded));
        assert!(!can_cancel_payment(PaymentStatus::Failed));
    }

    #[test]
    fn test_format_amount() {
        assert_eq!(format_amount(500, "PLN"), "5.00 zł");
        assert_eq!(format_amount(1000, "EUR"), "€10.00");
        assert_eq!(format_amount(2500, "USD"), "$25.00");
    }
}
