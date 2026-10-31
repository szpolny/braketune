use std::{env, fs, io, path::PathBuf};

fn entry_path() -> Option<PathBuf> {
    let config = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(config.join("autostart").join("braketune.desktop"))
}

/// Path to launch: the AppImage itself when packaged as one, else this binary.
fn exec_path() -> io::Result<PathBuf> {
    match env::var_os("APPIMAGE") {
        Some(path) => Ok(PathBuf::from(path)),
        None => env::current_exe(),
    }
}

pub fn is_enabled() -> bool {
    entry_path().is_some_and(|path| path.exists())
}

pub fn set_enabled(enabled: bool) -> io::Result<()> {
    let path = entry_path().ok_or_else(|| io::Error::other("no config directory"))?;
    if !enabled {
        return match fs::remove_file(&path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        };
    }
    fs::create_dir_all(path.parent().expect("entry has a parent"))?;
    let exec = exec_path()?;
    fs::write(
        path,
        format!(
            "[Desktop Entry]\nType=Application\nName=BrakeTune\nComment=Brake force control for Fanatec pedals\nExec=\"{}\" --minimized\nIcon=braketune\nTerminal=false\n",
            exec.display()
        ),
    )
}
