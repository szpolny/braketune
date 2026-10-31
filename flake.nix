{
  description = "BrakeTune development shell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ pkg-config nodejs pnpm cargo rustc clippy rustfmt ];
        buildInputs = with pkgs; [
          webkitgtk_4_1 gtk3 libsoup_3 librsvg openssl systemd glib-networking
        ];
        shellHook = ''
          export GIO_MODULE_DIR="${pkgs.glib-networking}/lib/gio/modules"
          export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS"
        '';
      };
    };
}
