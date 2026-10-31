//! Reads the physical pedals, scales the brake axis and republishes every
//! event through a virtual `uinput` device.

use std::{
    io,
    sync::{
        atomic::{AtomicU8, Ordering},
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

/// Pedal sets whose brake axis is rescaled in software by the Fanatec driver.
const PRODUCTS: &[(u16, &str)] = &[(0x183b, "ClubSport Pedals V3")];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub product_id: u16,
}

pub struct Mapper {
    percent: Arc<AtomicU8>,
    device: Arc<Mutex<Option<DeviceInfo>>>,
}

impl Mapper {
    pub fn new(percent: u8) -> Self {
        Self {
            percent: Arc::new(AtomicU8::new(percent)),
            device: Arc::new(Mutex::new(None)),
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
        thread::spawn(move || loop {
            if let Some((mut physical, info)) = find_pedals() {
                *device.lock().unwrap() = Some(info);
                on_change();
                // Errors (typically the pedals being unplugged) end the session.
                let _ = run_session(&mut physical, &percent);
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

fn run_session(physical: &mut Device, percent: &AtomicU8) -> io::Result<()> {
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
                    InputEvent::new(
                        e.event_type().0,
                        e.code(),
                        scaling::scale(e.value(), gain, min, max),
                    )
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
