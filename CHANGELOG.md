# Changelog

## [0.2.0](https://github.com/szpolny/braketune/compare/v0.1.0...v0.2.0) (2026-10-31)


### ⚠ BREAKING CHANGES

* lower brake force values now make the brake stronger instead of capping its output, so saved values behave differently.
* scale brake axis in software through a virtual device

### Features

* add brake force control app for Linux ([608bc1e](https://github.com/szpolny/braketune/commit/608bc1e23e01f47579fe218dce8d99eec83e9bfa))
* add Nix package and NixOS module ([856452a](https://github.com/szpolny/braketune/commit/856452a375e6973f26a22c5c5637bf0e454b8e41))
* add tray, autostart and 100% = full axis scaling ([ee37cc0](https://github.com/szpolny/braketune/commit/ee37cc021129fff56aa9ee4b7f169fc6859c3df9))
* brake force sets the pedal range needed for full braking ([67d19c1](https://github.com/szpolny/braketune/commit/67d19c145ed57b3fa04ab377a0fe9189a20fe680))
* live pedal meter, custom titlebar and new icon ([832d4f3](https://github.com/szpolny/braketune/commit/832d4f3658fbd2ab413ea816ce99d0f63e29e0d5))
* scale brake axis in software through a virtual device ([612bba3](https://github.com/szpolny/braketune/commit/612bba38dbf449c72f2f0644addf1e5d3337cde8))
* set brake force as 1-100% mapped to device levels ([2662692](https://github.com/szpolny/braketune/commit/26626921993cdfa86a37b7ebaade69b0abeb2039))


### Bug Fixes

* **dev:** disable WebKit DMA-BUF renderer in the dev shell ([a2352e2](https://github.com/szpolny/braketune/commit/a2352e235a4cc72a83a79cde53001fe1a5035c16))
