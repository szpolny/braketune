//! Reads the physical pedals, scales the brake axis and republishes every
//! event through a virtual `uinput` device.

use std::{
    io,
    sync::{
        atomic::{AtomicU16, AtomicU8, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use evdev::{
    uinput::VirtualDevice, AbsInfo, AbsoluteAxisCode, Device, EventType, InputEvent, UinputAbsSetup,
};
use serde::Serialize;

use crate::scaling;

pub const VENDOR_ID: u16 = 0x0eb7;
const VIRTUAL_NAME: &str = "BrakeTune Pedals";
const BRAKE_AXIS: AbsoluteAxisCode = AbsoluteAxisCode::ABS_Y;
/// Stored value of a fully pressed brake, see [`Pedal`].
const FULL: u16 = 10_000;

/// Pedal sets whose brake axis is rescaled in software by the Fanatec driver.
const PRODUCTS: &[(u16, &str)] = &[(0x183b, "ClubSport Pedals V3")];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub product_id: u16,
}

/// Live brake position in hundredths of a percent of the axis, so it fits an
/// atomic and survives being read from another thread mid-session.
#[derive(Default)]
struct Pedal {
    input: AtomicU16,
    output: AtomicU16,
}

/// Brake position as fractions of the full axis: `input` is how far the pedal
/// is pressed, `output` is what games see after scaling.
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct PedalReading {
    pub input: f32,
    pub output: f32,
}

pub struct Mapper {
    percent: Arc<AtomicU8>,
    device: Arc<Mutex<Option<DeviceInfo>>>,
    pedal: Arc<Pedal>,
}

impl Mapper {
    pub fn new(percent: u8) -> Self {
        Self {
            percent: Arc::new(AtomicU8::new(percent)),
            device: Arc::new(Mutex::new(None)),
            pedal: Arc::default(),
        }
    }

    pub fn pedal(&self) -> PedalReading {
        let read = |v: &AtomicU16| f32::from(v.load(Ordering::Relaxed)) / f32::from(FULL);
        PedalReading {
            input: read(&self.pedal.input),
            output: read(&self.pedal.output),
        }
    }

    pub fn set_percent(&self, percent: u8) {
        self.percent.store(percent, Ordering::Relaxed);
    }

    pub fn device(&self) -> Option<DeviceInfo> {
        self.device.lock().unwrap().clone()
    }

    /// Starts the background loop. `on_change` runs whenever the pedals are
    /// attached or removed.
    pub fn spawn(&self, on_change: impl Fn() + Send + 'static) {
        let percent = Arc::clone(&self.percent);
        let device = Arc::clone(&self.device);
        let pedal = Arc::clone(&self.pedal);
        thread::spawn(move || loop {
            if let Some((mut physical, info)) = find_pedals() {
                *device.lock().unwrap() = Some(info);
                on_change();
                // Errors (typically the pedals being unplugged) end the session.
                let _ = run_session(&mut physical, &percent, &pedal);
                pedal.input.store(0, Ordering::Relaxed);
                pedal.output.store(0, Ordering::Relaxed);
                *device.lock().unwrap() = None;
                on_change();
            }
            thread::sleep(Duration::from_secs(1));
        });
    }
}

fn find_pedals() -> Option<(Device, DeviceInfo)> {
    evdev::enumerate().find_map(|(_, dev)| {
        let id = dev.input_id();
        if id.vendor() != VENDOR_ID || dev.name() == Some(VIRTUAL_NAME) {
            return None;
        }
        let name = PRODUCTS
            .iter()
            .find(|(pid, _)| *pid == id.product())
            .map(|(_, name)| (*name).to_owned())?;
        let info = DeviceInfo {
            name,
            product_id: id.product(),
        };
        Some((dev, info))
    })
}

fn build_virtual(physical: &Device) -> io::Result<VirtualDevice> {
    let mut builder = VirtualDevice::builder()?
        .name(VIRTUAL_NAME)
        .input_id(physical.input_id());
    if let Some(keys) = physical.supported_keys() {
        builder = builder.with_keys(keys)?;
    }
    if let Ok(axes) = physical.get_absinfo() {
        for (code, info) in axes {
            builder = builder.with_absolute_axis(&UinputAbsSetup::new(code, info))?;
        }
    }
    builder.build()
}

fn brake_range(physical: &Device) -> io::Result<AbsInfo> {
    physical
        .get_absinfo()?
        .find(|(code, _)| *code == BRAKE_AXIS)
        .map(|(_, info)| info)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "brake axis missing"))
}

/// Position of `value` within `min..=max` in units of [`FULL`].
fn fraction(value: i32, min: i32, max: i32) -> u16 {
    if max <= min {
        return 0;
    }
    let share = (value - min) as f32 / (max - min) as f32;
    (share.clamp(0.0, 1.0) * f32::from(FULL)).round() as u16
}

fn run_session(physical: &mut Device, percent: &AtomicU8, pedal: &Pedal) -> io::Result<()> {
    let range = brake_range(physical)?;
    let (min, max) = (range.minimum(), range.maximum());
    let mut virtual_dev = build_virtual(physical)?;
    physical.grab()?;

    loop {
        let gain = scaling::gain(percent.load(Ordering::Relaxed));
        let batch: Vec<InputEvent> = physical
            .fetch_events()?
            .filter(|e| e.event_type() != EventType::SYNCHRONIZATION)
            .map(|e| {
                if e.event_type() == EventType::ABSOLUTE && e.code() == BRAKE_AXIS.0 {
                    let scaled = scaling::scale(e.value(), gain, min, max);
                    pedal
                        .input
                        .store(fraction(e.value(), min, max), Ordering::Relaxed);
                    pedal
                        .output
                        .store(fraction(scaled, min, max), Ordering::Relaxed);
                    InputEvent::new(e.event_type().0, e.code(), scaled)
                } else {
                    e
                }
            })
            .collect();
        if !batch.is_empty() {
            virtual_dev.emit(&batch)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fraction_spans_the_axis() {
        assert_eq!(fraction(0, 0, 4095), 0);
        assert_eq!(fraction(4095, 0, 4095), FULL);
        assert_eq!(fraction(60, 10, 110), FULL / 2);
    }

    #[test]
    fn fraction_clamps_and_handles_empty_ranges() {
        assert_eq!(fraction(-5, 0, 100), 0);
        assert_eq!(fraction(200, 0, 100), FULL);
        assert_eq!(fraction(5, 5, 5), 0);
    }
}
