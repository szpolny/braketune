{
  lib,
  rustPlatform,
  pkg-config,
  makeWrapper,
  fontconfig,
  freetype,
  libxkbcommon,
  wayland,
  vulkan-loader,
  libGL,
  libxcb,
  libx11,
  libxcursor,
  libxi,
  libxrandr,
}:

rustPlatform.buildRustPackage {
  pname = "braketune";
  version = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../assets
      ../src
      ../packaging
    ];
  };
  cargoLock.lockFile = ../Cargo.lock;
  nativeBuildInputs = [ pkg-config makeWrapper ];
  buildInputs = [ fontconfig freetype libxkbcommon wayland vulkan-loader libGL libxcb ];
  postInstall = ''
    install -Dm444 packaging/udev/70-braketune.rules $out/lib/udev/rules.d/70-braketune.rules
    install -Dm444 packaging/braketune.desktop $out/share/applications/braketune.desktop
    install -Dm444 assets/icon.svg $out/share/icons/hicolor/scalable/apps/braketune.svg
    install -Dm444 assets/icon.png $out/share/icons/hicolor/1024x1024/apps/braketune.png
    wrapProgram $out/bin/braketune --set BRAKETUNE_EXECUTABLE $out/bin/braketune --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [
      libxkbcommon wayland vulkan-loader libGL libx11 libxcursor libxi libxrandr
    ]}
  '';
  meta = {
    description = "Brake force control for Fanatec pedals";
    homepage = "https://github.com/szpolny/braketune";
    license = lib.licenses.mit;
    platforms = [ "x86_64-linux" ];
    mainProgram = "braketune";
  };
}
