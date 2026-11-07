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
        packages = with pkgs; [ cargo rustc clippy rustfmt ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
          libxkbcommon wayland vulkan-loader libGL
          libx11 libxcursor libxi libxrandr
        ]);
      };
    };
}
