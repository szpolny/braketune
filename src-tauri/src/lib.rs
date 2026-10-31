mod mapper;
mod scaling;
mod settings;

use std::{path::PathBuf, sync::Mutex};

use mapper::{DeviceInfo, Mapper};
use serde::Serialize;
use settings::Settings;
use tauri::{AppHandle, Emitter, Manager, State};

struct AppState {
    settings: Mutex<Settings>,
    settings_path: PathBuf,
    mapper: Mapper,
}

#[derive(Serialize, Clone)]
struct Status {
    device: Option<DeviceInfo>,
    settings: Settings,
}

impl AppState {
    fn status(&self) -> Status {
        Status {
            device: self.mapper.device(),
            settings: self.settings.lock().unwrap().clone(),
        }
    }
}

#[tauri::command]
fn get_status(state: State<AppState>) -> Status {
    state.status()
}

#[tauri::command]
fn set_brake_force(percent: u8, state: State<AppState>) -> Result<Status, String> {
    if !(scaling::MIN_PERCENT..=scaling::MAX_PERCENT).contains(&percent) {
        return Err(format!(
            "brake force must be between {}% and {}%",
            scaling::MIN_PERCENT,
            scaling::MAX_PERCENT
        ));
    }
    {
        let mut settings = state.settings.lock().unwrap();
        settings.brake_force = percent;
        settings
            .save(&state.settings_path)
            .map_err(|e| e.to_string())?;
    }
    state.mapper.set_percent(percent);
    Ok(state.status())
}

fn notify(app: &AppHandle) {
    let _ = app.emit("status-changed", app.state::<AppState>().status());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let settings_path = app.path().app_config_dir()?.join("settings.json");
            let settings = Settings::load(&settings_path);
            let mapper = Mapper::new(settings.brake_force);
            let handle = app.handle().clone();
            mapper.spawn(move || notify(&handle));
            app.manage(AppState {
                settings: Mutex::new(settings),
                settings_path,
                mapper,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_status, set_brake_force])
        .run(tauri::generate_context!())
        .expect("error while running BrakeTune");
}
