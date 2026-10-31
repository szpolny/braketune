//! Brake force scaling.
//!
//! ClubSport V3 pedals connected over USB expose no device-side brake force
//! setting: the Windows driver rescales the brake axis in software. BrakeTune
//! does the same. Per Fanatec's manual the strongest setting scales the brake
//! signal by a factor of three.

pub const MIN_PERCENT: u8 = 1;
pub const MAX_PERCENT: u8 = 100;
const MAX_GAIN: f32 = 3.0;

/// Gain applied to the brake axis: 1.0 at 1% (unmodified) up to 3.0 at 100%.
pub fn gain(percent: u8) -> f32 {
    let p = percent.clamp(MIN_PERCENT, MAX_PERCENT);
    1.0 + (MAX_GAIN - 1.0) * f32::from(p - MIN_PERCENT) / f32::from(MAX_PERCENT - MIN_PERCENT)
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
    fn gain_spans_one_to_three() {
        assert_eq!(gain(MIN_PERCENT), 1.0);
        assert_eq!(gain(MAX_PERCENT), 3.0);
        assert!(gain(50) > 1.9 && gain(50) < 2.1);
    }

    #[test]
    fn gain_clamps_out_of_range_input() {
        assert_eq!(gain(0), 1.0);
        assert_eq!(gain(250), 3.0);
    }

    #[test]
    fn unity_gain_is_a_passthrough() {
        for raw in [0, 1, 500, 4095] {
            assert_eq!(scale(raw, 1.0, 0, 4095), raw);
        }
    }

    #[test]
    fn scaling_saturates_at_axis_max() {
        assert_eq!(scale(1000, 2.0, 0, 4095), 2000);
        assert_eq!(scale(3000, 3.0, 0, 4095), 4095);
        assert_eq!(scale(0, 3.0, 0, 4095), 0);
    }

    #[test]
    fn scaling_is_relative_to_axis_minimum() {
        assert_eq!(scale(10, 2.0, 10, 100), 10);
        assert_eq!(scale(20, 2.0, 10, 100), 30);
    }
}
