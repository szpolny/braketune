mod autostart;
mod desktop;
mod mapper;
mod scaling;
mod settings;
mod ui;

fn main() {
    if let Err(error) = desktop::run() {
        eprintln!("BrakeTune: {error}");
        std::process::exit(1);
    }
}
