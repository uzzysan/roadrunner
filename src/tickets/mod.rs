pub mod qr;

use crate::models::ticket::TicketType;

/// Cennik biletów
pub struct TicketPricing;

impl TicketPricing {
    /// Zwraca cenę biletu w groszach
    pub fn get_price(ticket_type: TicketType) -> i32 {
        match ticket_type {
            TicketType::Single => 500,     // 5.00 PLN
            TicketType::Weekly => 2500,    // 25.00 PLN
            TicketType::Monthly => 8000,   // 80.00 PLN
            TicketType::Discounted => 250, // 2.50 PLN
        }
    }

    /// Zwraca okres ważności w dniach
    pub fn get_validity_days(ticket_type: TicketType) -> i64 {
        match ticket_type {
            TicketType::Single => 1,
            TicketType::Weekly => 7,
            TicketType::Monthly => 30,
            TicketType::Discounted => 1,
        }
    }

    /// Zwraca nazwę biletu
    pub fn get_name(ticket_type: TicketType) -> &'static str {
        match ticket_type {
            TicketType::Single => "Bilet jednorazowy",
            TicketType::Weekly => "Bilet tygodniowy",
            TicketType::Monthly => "Bilet miesięczny",
            TicketType::Discounted => "Bilet ulgowy",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_pricing() {
        assert_eq!(TicketPricing::get_price(TicketType::Single), 500);
        assert_eq!(TicketPricing::get_price(TicketType::Weekly), 2500);
        assert_eq!(TicketPricing::get_price(TicketType::Monthly), 8000);
        assert_eq!(TicketPricing::get_price(TicketType::Discounted), 250);
    }

    #[test]
    fn test_validity_days() {
        assert_eq!(TicketPricing::get_validity_days(TicketType::Single), 1);
        assert_eq!(TicketPricing::get_validity_days(TicketType::Weekly), 7);
        assert_eq!(TicketPricing::get_validity_days(TicketType::Monthly), 30);
    }
}
