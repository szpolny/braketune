<p align="center"><img src="assets/icon.svg" width="96" alt="BrakeTune"></p>

# BrakeTune

The Fanatec control panel doesn't exist on Linux. BrakeTune brings its **brake force** setting (1–100%) to Linux, the same way the Windows driver does it: by scaling the brake axis in software.

## How it works

BrakeTune reads your pedals, scales the brake axis (100% is the unmodified axis; at 20%, a fifth of the pedal's range already gives full braking) and republishes everything as a virtual controller named **BrakeTune Pedals**. Pick that controller in your game or simulator. The original pedals are hidden while BrakeTune runs, and closing the window keeps it running in the tray.

## Supported pedals

ClubSport Pedals V3 / V3 Inverted, connected to the PC over USB.

## Install

Grab a `.deb`, `.rpm` or `.AppImage` from [Releases](https://github.com/szpolny/braketune/releases/latest). The packages install a udev rule that lets your user create the virtual device; with the AppImage, copy [`70-braketune.rules`](src-tauri/udev/70-braketune.rules) to `/etc/udev/rules.d/`, load the `uinput` module and replug the pedals.

### Nix

```sh
nix run github:szpolny/braketune
```

On NixOS, add the flake's `nixosModules.default` to install the app, its udev rule and `uinput` support. Pin a release with `github:szpolny/braketune/v<version>`.

## Development

Requires Rust, Node.js and [pnpm](https://pnpm.io). On NixOS, `nix develop` provides everything.

```sh
pnpm install
pnpm tauri dev      # run
pnpm tauri build    # package
```

Commits follow [Conventional Commits](https://www.conventionalcommits.org); releases are versioned and published automatically.

## License

[MIT](LICENSE)
