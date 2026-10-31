mod autostart;
mod mapper;
mod scaling;
mod settings;

use std::{path::PathBuf, sync::Mutex};

use mapper::{DeviceInfo, Mapper};
use serde::Serialize;
use settings::Settings;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};

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

#[tauri::command]
fn get_autostart() -> bool {
    autostart::is_enabled()
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<bool, String> {
    autostart::set_enabled(enabled).map_err(|e| e.to_string())?;
    Ok(autostart::is_enabled())
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show BrakeTune", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("BrakeTune")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn notify(app: &AppHandle) {
    let _ = app.emit("status-changed", app.state::<AppState>().status());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show_window(app)
        }))
        .on_window_event(|window, event| {
            // Closing the window keeps the pedal mapper running in the tray.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
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
            build_tray(app)?;
            if std::env::args().any(|arg| arg == "--minimized") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            set_brake_force,
            get_autostart,
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("error while running BrakeTune");
}
