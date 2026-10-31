use hidapi::HidApi;
use serde::Serialize;

use crate::protocol::{self, VENDOR_ID};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub product_id: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("no supported Fanatec pedals found")]
    NotFound,
    #[error(transparent)]
    Protocol(#[from] protocol::ProtocolError),
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),
}

/// Lists the first supported pedal set currently attached.
pub fn detect(api: &HidApi) -> Option<(DeviceInfo, std::ffi::CString)> {
    api.device_list()
        .filter(|d| d.vendor_id() == VENDOR_ID)
        .find_map(|d| {
            protocol::product_name(d.product_id()).map(|name| {
                (
                    DeviceInfo {
                        name: name.to_owned(),
                        product_id: d.product_id(),
                    },
                    d.path().to_owned(),
                )
            })
        })
}

pub fn set_brake_force(api: &HidApi, level: u8) -> Result<DeviceInfo, DeviceError> {
    let report = protocol::brake_force_report(level)?;
    let (info, path) = detect(api).ok_or(DeviceError::NotFound)?;
    api.open_path(&path)?.write(&report)?;
    Ok(info)
}
