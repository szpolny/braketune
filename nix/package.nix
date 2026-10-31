{
  lib,
  rustPlatform,
  fetchPnpmDeps,
  pnpmConfigHook,
  pnpm,
  nodejs,
  cargo-tauri,
  pkg-config,
  wrapGAppsHook3,
  webkitgtk_4_1,
  openssl,
  glib-networking,
  libayatana-appindicator,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "braketune";
  version = (builtins.fromJSON (builtins.readFile ../package.json)).version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../assets/icon.svg
      ../package.json
      ../pnpm-lock.yaml
      ../index.html
      ../vite.config.ts
      ../svelte.config.js
      ../tsconfig.json
      ../src
      ../src-tauri
    ];
  };

  cargoRoot = "src-tauri";
  buildAndTestSubdir = "src-tauri";
  cargoLock.lockFile = ../src-tauri/Cargo.lock;

  pnpmDeps = fetchPnpmDeps {
    inherit (finalAttrs) pname version src;
    inherit pnpm;
    fetcherVersion = 5;
    hash = "sha256-hnVWMYqEL6271mUrosYDF9NNof5wL42aGPWhSg38Xcw=";
  };

  nativeBuildInputs = [
    cargo-tauri.hook
    pnpmConfigHook
    pnpm
    nodejs
    pkg-config
    wrapGAppsHook3
  ];

  buildInputs = [
    webkitgtk_4_1
    openssl
    glib-networking
    libayatana-appindicator
  ];

  preFixup = ''
    gappsWrapperArgs+=(--prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ libayatana-appindicator ]})
  '';

  postInstall = ''
    install -Dm444 src-tauri/udev/70-braketune.rules \
      $out/lib/udev/rules.d/70-braketune.rules
  '';

  meta = {
    description = "Brake force control for Fanatec pedals";
    homepage = "https://github.com/szpolny/braketune";
    license = lib.licenses.mit;
    platforms = [ "x86_64-linux" ];
    mainProgram = "braketune";
  };
})
