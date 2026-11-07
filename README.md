<p align="center"><img src="assets/icon.svg" width="96" alt="BrakeTune"></p>

# BrakeTune

The Fanatec control panel doesn't exist on Linux. BrakeTune brings its **brake force** setting (1–100%) to Linux, the same way the Windows driver does it: by scaling the brake axis in software.

## How it works

BrakeTune reads your pedals, scales the brake axis (100% is the unmodified axis; at 20%, a fifth of the pedal's range already gives full braking) and republishes everything as a virtual controller named **BrakeTune Pedals**. Pick that controller in your game or simulator. The original pedals are hidden while BrakeTune runs, and closing the window keeps it running in the tray.

## Supported pedals

ClubSport Pedals V3 / V3 Inverted, connected to the PC over USB.

## Install

Grab a `.deb`, `.rpm` or `.AppImage` from [Releases](https://github.com/szpolny/braketune/releases/latest). The packages install a udev rule that lets your user create the virtual device; with the AppImage, copy [`70-braketune.rules`](packaging/udev/70-braketune.rules) to `/etc/udev/rules.d/`, load the `uinput` module and replug the pedals.

### Nix

```sh
nix run github:szpolny/braketune
```

On NixOS, add the flake's `nixosModules.default` to install the app, its udev rule and `uinput` support. Pin a release with `github:szpolny/braketune/v<version>`.

## Development

BrakeTune uses [GPUI](https://gpui.rs/) for its native Rust UI, with Wayland and X11 support. It does not require a webview or Node.js. On NixOS, `nix develop` provides the Rust toolchain and native libraries. A working Vulkan driver is required.

```sh
nix develop
cargo run                  # run
cargo run -- --minimized    # start in the tray
cargo build --release      # build
cargo test                 # test pedal scaling and settings
cargo clippy --all-targets -- -D warnings
```

On other Linux distributions, install Rust and the Fontconfig, FreeType, xkbcommon (including X11), Wayland and Vulkan development libraries. See the CI workflow for Ubuntu package names.

Closing the window keeps the mapper running when a StatusNotifier tray is available. Reopen it from the tray or by launching BrakeTune again. Without tray support, closing the window quits. Existing Tauri settings under `$XDG_CONFIG_HOME/com.szpolny.braketune/settings.json` (default `~/.config/com.szpolny.braketune/settings.json`) are preserved.

The release workflow builds `.deb` and `.rpm` packages with nFPM and an AppImage with linuxdeploy. `nix build` builds the native package, desktop entry, icon and udev rules.

Commits follow [Conventional Commits](https://www.conventionalcommits.org); releases are versioned and published automatically.

## License

[MIT](LICENSE)
