//! Fanatec pedal wire protocol.
//!
//! The command layout follows the community-documented protocol used by the
//! `hid-fanatecff` Linux driver. Reports are 7 data bytes behind report id 0.

use thiserror::Error;

pub const VENDOR_ID: u16 = 0x0eb7;

/// USB product ids of pedal sets that accept the brake load-cell command.
pub const PEDAL_PRODUCTS: &[(u16, &str)] = &[
    (0x183b, "ClubSport Pedals V3"),
    (0x6204, "CSL Elite Pedals"),
    (0x6205, "CSL Pedals Load Cell"),
    (0x6206, "CSL Pedals LC V2"),
];

pub const MIN_BRAKE_FORCE: u8 = 1;
pub const MAX_BRAKE_FORCE: u8 = 10;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("brake force must be between {MIN_BRAKE_FORCE} and {MAX_BRAKE_FORCE}, got {0}")]
    BrakeForceOutOfRange(u8),
}

pub fn product_name(product_id: u16) -> Option<&'static str> {
    PEDAL_PRODUCTS
        .iter()
        .find(|(pid, _)| *pid == product_id)
        .map(|(_, name)| *name)
}

/// Builds the output report that sets the brake load-cell force level.
pub fn brake_force_report(level: u8) -> Result<[u8; 8], ProtocolError> {
    if !(MIN_BRAKE_FORCE..=MAX_BRAKE_FORCE).contains(&level) {
        return Err(ProtocolError::BrakeForceOutOfRange(level));
    }
    Ok([0x00, 0xf8, 0x09, 0x01, 0x16, level, 0x00, 0x00])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_report_for_valid_levels() {
        assert_eq!(
            brake_force_report(5).unwrap(),
            [0x00, 0xf8, 0x09, 0x01, 0x16, 5, 0, 0]
        );
        assert!(brake_force_report(MIN_BRAKE_FORCE).is_ok());
        assert!(brake_force_report(MAX_BRAKE_FORCE).is_ok());
    }

    #[test]
    fn rejects_out_of_range_levels() {
        assert_eq!(
            brake_force_report(0),
            Err(ProtocolError::BrakeForceOutOfRange(0))
        );
        assert_eq!(
            brake_force_report(11),
            Err(ProtocolError::BrakeForceOutOfRange(11))
        );
    }

    #[test]
    fn resolves_product_names() {
        assert_eq!(product_name(0x183b), Some("ClubSport Pedals V3"));
        assert_eq!(product_name(0x0001), None);
    }
}
