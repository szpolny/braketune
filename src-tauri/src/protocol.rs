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

/// The device accepts ten discrete load-cell levels. Anything outside this
/// range can leave the pedals unresponsive until they are replugged.
pub const MIN_BRAKE_LEVEL: u8 = 1;
pub const MAX_BRAKE_LEVEL: u8 = 10;

/// Brake force is exposed to users as a percentage, like the official app.
pub const MIN_BRAKE_PERCENT: u8 = 1;
pub const MAX_BRAKE_PERCENT: u8 = 100;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("brake force must be between {MIN_BRAKE_PERCENT}% and {MAX_BRAKE_PERCENT}%, got {0}%")]
    BrakeForceOutOfRange(u8),
}

pub fn product_name(product_id: u16) -> Option<&'static str> {
    PEDAL_PRODUCTS
        .iter()
        .find(|(pid, _)| *pid == product_id)
        .map(|(_, name)| *name)
}

/// Maps a brake force percentage (1-100) onto the device's ten levels.
pub fn percent_to_level(percent: u8) -> Result<u8, ProtocolError> {
    if !(MIN_BRAKE_PERCENT..=MAX_BRAKE_PERCENT).contains(&percent) {
        return Err(ProtocolError::BrakeForceOutOfRange(percent));
    }
    Ok(percent.div_ceil(10).clamp(MIN_BRAKE_LEVEL, MAX_BRAKE_LEVEL))
}

/// Builds the output report that sets the brake load-cell force.
pub fn brake_force_report(percent: u8) -> Result<[u8; 8], ProtocolError> {
    let level = percent_to_level(percent)?;
    Ok([0x00, 0xf8, 0x09, 0x01, 0x16, level, 0x00, 0x00])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_percent_to_ten_levels() {
        assert_eq!(percent_to_level(1), Ok(1));
        assert_eq!(percent_to_level(10), Ok(1));
        assert_eq!(percent_to_level(11), Ok(2));
        assert_eq!(percent_to_level(50), Ok(5));
        assert_eq!(percent_to_level(100), Ok(10));
    }

    #[test]
    fn never_emits_a_level_outside_the_device_range() {
        for percent in MIN_BRAKE_PERCENT..=MAX_BRAKE_PERCENT {
            let report = brake_force_report(percent).unwrap();
            assert!((MIN_BRAKE_LEVEL..=MAX_BRAKE_LEVEL).contains(&report[5]));
        }
    }

    #[test]
    fn builds_report() {
        assert_eq!(
            brake_force_report(50).unwrap(),
            [0x00, 0xf8, 0x09, 0x01, 0x16, 5, 0, 0]
        );
    }

    #[test]
    fn rejects_out_of_range_percent() {
        assert_eq!(
            brake_force_report(0),
            Err(ProtocolError::BrakeForceOutOfRange(0))
        );
        assert_eq!(
            brake_force_report(101),
            Err(ProtocolError::BrakeForceOutOfRange(101))
        );
    }

    #[test]
    fn resolves_product_names() {
        assert_eq!(product_name(0x183b), Some("ClubSport Pedals V3"));
        assert_eq!(product_name(0x0001), None);
    }
}
