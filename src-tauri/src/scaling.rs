//! Brake force scaling.
//!
//! ClubSport V3 pedals connected over USB expose no device-side brake force
//! setting: the Windows driver rescales the brake axis in software. BrakeTune
//! does the same: the setting is how much of the pedal's range is needed for
//! full braking, so 100% is the unmodified axis and lower values brake harder.

pub const MIN_PERCENT: u8 = 1;
pub const MAX_PERCENT: u8 = 100;

/// Gain applied to the brake axis: 1.0 (unmodified) at 100%, 5.0 at 20%
/// (a fifth of the pedal reaches full braking), 100.0 at 1%.
pub fn gain(percent: u8) -> f32 {
    f32::from(MAX_PERCENT) / f32::from(percent.clamp(MIN_PERCENT, MAX_PERCENT))
}

/// Scales `raw` (relative to the axis minimum) and clamps it to the axis range.
pub fn scale(raw: i32, gain: f32, min: i32, max: i32) -> i32 {
    let scaled = (raw - min) as f32 * gain;
    (min as f32 + scaled).round().clamp(min as f32, max as f32) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_is_the_inverse_of_the_percentage() {
        assert_eq!(gain(MAX_PERCENT), 1.0);
        assert_eq!(gain(50), 2.0);
        assert_eq!(gain(20), 5.0);
        assert_eq!(gain(MIN_PERCENT), 100.0);
    }

    #[test]
    fn gain_clamps_out_of_range_input() {
        assert_eq!(gain(0), 100.0);
        assert_eq!(gain(250), 1.0);
    }

    #[test]
    fn full_force_is_a_passthrough() {
        for raw in [0, 1, 500, 4095] {
            assert_eq!(scale(raw, gain(MAX_PERCENT), 0, 4095), raw);
        }
    }

    #[test]
    fn reduced_force_saturates_early() {
        assert_eq!(scale(819, gain(20), 0, 4095), 4095);
        assert_eq!(scale(400, gain(20), 0, 4095), 2000);
        assert_eq!(scale(4095, gain(20), 0, 4095), 4095);
        assert_eq!(scale(0, gain(20), 0, 4095), 0);
    }

    #[test]
    fn scaling_is_relative_to_axis_minimum() {
        assert_eq!(scale(10, 2.0, 10, 110), 10);
        assert_eq!(scale(30, 2.0, 10, 110), 50);
        assert_eq!(scale(90, 2.0, 10, 110), 110);
    }
}
