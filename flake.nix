{
  description = "BrakeTune: brake force control for Fanatec pedals";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      package = pkgs.callPackage ./nix/package.nix { };
    in {
      packages.${system} = {
        default = package;
        braketune = package;
      };

      nixosModules.default = { pkgs, ... }: {
        environment.systemPackages = [ self.packages.${pkgs.system}.default ];
        hardware.uinput.enable = true;
        services.udev.packages = [ self.packages.${pkgs.system}.default ];
      };

      devShells.${system}.default = pkgs.mkShell {
        inputsFrom = [ package ];
        packages = with pkgs; [ clippy rustfmt librsvg ];
        shellHook = ''
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.libayatana-appindicator ]}:$LD_LIBRARY_PATH"
          # WebKitGTK's DMA-BUF renderer crashes dev builds with "Error 71
          # (Protocol error)" on Wayland with NVIDIA drivers.
          export WEBKIT_DISABLE_DMABUF_RENDERER=1
          export GIO_MODULE_DIR="${pkgs.glib-networking}/lib/gio/modules"
          export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS"
        '';
      };
    };
}
