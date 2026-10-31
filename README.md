<p align="center"><img src="assets/icon.svg" width="96" alt="BrakeTune"></p>

# BrakeTune

A small, fast alternative to the Fanatec control panel for one job: setting the **brake force** of your pedals. Built with Tauri, Rust and Svelte.

## Features

- Detects Fanatec pedals over USB and sets brake load-cell force (1–10)
- Remembers your setting and re-applies it whenever the pedals connect
- Tiny native app, no Fanatec software required

## Supported pedals

ClubSport Pedals V3 / V3 Inverted, CSL Elite Pedals, CSL Pedals Load Cell (incl. LC V2).

> The command comes from the community-documented protocol of the [hid-fanatecff](https://github.com/gotzl/hid-fanatecff) driver, not from Fanatec. Unofficial and used at your own risk.

## Install

Download the installer for Windows from [Releases](https://github.com/szpolny/braketune/releases/latest).

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
