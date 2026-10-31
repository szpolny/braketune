mod device;
mod protocol;
mod settings;

use std::{path::PathBuf, sync::Mutex, thread, time::Duration};

use device::DeviceInfo;
use hidapi::HidApi;
use serde::Serialize;
use settings::Settings;
use tauri::{AppHandle, Emitter, Manager, State};

struct AppState {
    settings: Mutex<Settings>,
    settings_path: PathBuf,
    hid: Mutex<HidApi>,
    device: Mutex<Option<DeviceInfo>>,
}

#[derive(Serialize, Clone)]
struct Status {
    device: Option<DeviceInfo>,
    settings: Settings,
}

impl AppState {
    fn status(&self) -> Status {
        Status {
            device: self.device.lock().unwrap().clone(),
            settings: self.settings.lock().unwrap().clone(),
        }
    }

    fn persist(&self) -> Result<(), String> {
        self.settings
            .lock()
            .unwrap()
            .save(&self.settings_path)
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_status(state: State<AppState>) -> Status {
    state.status()
}

#[tauri::command]
fn set_brake_force(level: u8, state: State<AppState>) -> Result<Status, String> {
    {
        let api = state.hid.lock().unwrap();
        device::set_brake_force(&api, level).map_err(|e| e.to_string())?;
    }
    state.settings.lock().unwrap().brake_force = level;
    state.persist()?;
    Ok(state.status())
}

#[tauri::command]
fn set_apply_on_connect(enabled: bool, state: State<AppState>) -> Result<Status, String> {
    state.settings.lock().unwrap().apply_on_connect = enabled;
    state.persist()?;
    Ok(state.status())
}

/// Watches for pedals being plugged in or removed and re-applies the saved
/// brake force on connect, since the pedals forget it when power cycled.
fn spawn_watcher(app: AppHandle) {
    thread::spawn(move || loop {
        let state = app.state::<AppState>();
        let found = {
            let mut api = state.hid.lock().unwrap();
            let _ = api.refresh_devices();
            device::detect(&api).map(|(info, _)| info)
        };

        let changed = {
            let mut current = state.device.lock().unwrap();
            let changed = *current != found;
            *current = found.clone();
            changed
        };

        if changed {
            let (apply, level) = {
                let s = state.settings.lock().unwrap();
                (s.apply_on_connect, s.brake_force)
            };
            if found.is_some() && apply {
                let api = state.hid.lock().unwrap();
                let _ = device::set_brake_force(&api, level);
            }
            let _ = app.emit("status-changed", state.status());
        }
        thread::sleep(Duration::from_secs(2));
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let settings_path = app.path().app_config_dir()?.join("settings.json");
            app.manage(AppState {
                settings: Mutex::new(Settings::load(&settings_path)),
                settings_path,
                hid: Mutex::new(HidApi::new()?),
                device: Mutex::new(None),
            });
            spawn_watcher(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            set_brake_force,
            set_apply_on_connect
        ])
        .run(tauri::generate_context!())
        .expect("error while running BrakeTune");
}
