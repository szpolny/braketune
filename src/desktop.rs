//! Desktop lifetime is independent of the window: the mapper survives closing it.
use std::{
    cell::Cell,
    cell::RefCell,
    fs::{self, File, OpenOptions},
    io,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    rc::Rc,
    sync::mpsc::{self, Sender},
    time::Duration,
};

use fs2::FileExt;
use gpui::{
    App, AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowDecorations,
    WindowHandle, WindowOptions, px, size,
};
use ksni::blocking::TrayMethods;

use crate::ui::{Model, Panel, SharedModel};

pub fn config_dir() -> io::Result<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|dir| dir.join("com.szpolny.braketune"))
        .ok_or_else(|| io::Error::other("no config directory"))
}

// The lock protects socket replacement, including recovery after a crash.
struct Instance {
    _lock: File,
    listener: UnixListener,
    path: PathBuf,
}

impl Instance {
    fn acquire() -> io::Result<Option<Self>> {
        let dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .map(Ok)
            .unwrap_or_else(config_dir)?;
        Self::acquire_in(dir)
    }

    fn acquire_in(dir: PathBuf) -> io::Result<Option<Self>> {
        fs::create_dir_all(&dir)?;
        let path = dir.join("braketune.sock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.join("braketune.lock"))?;
        match lock.try_lock_exclusive() {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                // Allow a first instance a short interval to finish binding.
                for _ in 0..20 {
                    if UnixStream::connect(&path).is_ok() {
                        return Ok(None);
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
                return Err(io::Error::other("another instance is starting; try again"));
            }
            Err(error) => return Err(error),
        }
        match fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
        let listener = UnixListener::bind(&path)?;
        listener.set_nonblocking(true)?;
        Ok(Some(Self {
            _lock: lock,
            listener,
            path,
        }))
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Clone, Copy)]
enum Command {
    Show,
    Quit,
}
struct Tray {
    tx: Sender<Command>,
}
impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "braketune".into()
    }
    fn title(&self) -> String {
        "BrakeTune".into()
    }
    fn icon_name(&self) -> String {
        "braketune".into()
    }
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let image = image::load_from_memory(include_bytes!("../assets/tray.png"))
            .expect("embedded icon")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let argb = image
            .pixels()
            .flat_map(|p| [p[3], p[0], p[1], p[2]])
            .collect();
        vec![ksni::Icon {
            width: width as i32,
            height: height as i32,
            data: argb,
        }]
    }
    fn activate(&mut self, _: i32, _: i32) {
        let _ = self.tx.send(Command::Show);
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::StandardItem;
        vec![
            StandardItem {
                label: "Show BrakeTune".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(Command::Show);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(Command::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn show(panel: &mut Option<WindowHandle<Panel>>, model: &SharedModel, cx: &mut App) {
    if let Some(handle) = panel
        && handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    {
        return;
    }
    let model = model.clone();
    match cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(400.), px(580.)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("BrakeTune".into()),
                ..Default::default()
            }),
            window_decorations: Some(WindowDecorations::Client),
            app_id: Some("braketune".into()),
            is_resizable: false,
            ..Default::default()
        },
        |window, cx| cx.new(|cx| Panel::new(model, window, cx)),
    ) {
        Ok(handle) => *panel = Some(handle),
        Err(error) => eprintln!("Unable to open BrakeTune: {error}"),
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let Some(instance) = Instance::acquire()? else {
        return Ok(());
    };
    let instance = Rc::new(instance);
    let (tx, rx) = mpsc::channel();
    let rx = Rc::new(rx);
    let _tray = Tray { tx }.spawn();
    let tray_available = _tray.is_ok();
    if let Err(error) = &_tray {
        eprintln!("Tray unavailable: {error}. Closing the window will quit.");
    }
    let model = Rc::new(RefCell::new(Model::new(
        config_dir()?.join("settings.json"),
        tray_available,
    )));
    model.borrow().start();
    let quitting = Rc::new(Cell::new(false));
    let mut open = !tray_available || !std::env::args().any(|arg| arg == "--minimized");
    loop {
        if open {
            {
                let (model, instance, rx, quitting) = (
                    model.clone(),
                    instance.clone(),
                    rx.clone(),
                    quitting.clone(),
                );
                // GPUI's Linux event loop ends when its final window closes. Keep the
                // mapper and tray outside it and start a new UI session when requested.
                Application::new().run(move |cx| {
                    let mut panel = None;
                    show(&mut panel, &model, cx);
                    if panel.is_none() {
                        quitting.set(true);
                        cx.quit();
                        return;
                    }
                    cx.spawn(async move |cx| {
                        loop {
                            cx.background_executor()
                                .timer(Duration::from_millis(16))
                                .await;
                            if cx
                                .update(|cx| {
                                    while instance.listener.accept().is_ok() {
                                        show(&mut panel, &model, cx);
                                    }
                                    for command in rx.try_iter() {
                                        match command {
                                            Command::Show => show(&mut panel, &model, cx),
                                            Command::Quit => {
                                                quitting.set(true);
                                                cx.quit();
                                            }
                                        }
                                    }
                                    if model.borrow_mut().refresh(true)
                                        && let Some(handle) = &panel
                                    {
                                        let _ = handle.update(cx, |_, _, cx| cx.notify());
                                    }
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                    })
                    .detach();
                });
            }
            model.borrow_mut().save();
            if quitting.get() || !tray_available {
                break;
            }
        }
        open = false;
        while instance.listener.accept().is_ok() {
            open = true;
        }
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Command::Quit) => break,
            Ok(Command::Show) => open = true,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_launch_signals_owner_and_stale_socket_is_recovered() {
        let dir = std::env::temp_dir().join(format!("braketune-instance-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let first = Instance::acquire_in(dir.clone()).unwrap().unwrap();
        assert!(Instance::acquire_in(dir.clone()).unwrap().is_none());
        assert!(first.listener.accept().is_ok());
        let path = first.path.clone();
        drop(first);
        assert!(!path.exists());
        // A socket left by an abruptly terminated process must not prevent startup.
        drop(UnixListener::bind(&path).unwrap());
        let recovered = Instance::acquire_in(dir.clone()).unwrap().unwrap();
        drop(recovered);
        fs::remove_dir_all(dir).unwrap();
    }
}
