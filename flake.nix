{
  description = "Home Manager configuration for kofron";

  inputs = {
    # stable is the default for most packages (25.05 to match HM)
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

    # pull in unstable so we can grab newer apps (e.g. zed)
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";

    # upstream Zed flake
    zed.url = "github:zed-industries/zed";

    home-manager = {
      # use 25.05 so we have `programs.ghostty` and other newer modules
      url = "github:nix-community/home-manager/release-25.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ nixpkgs, nixpkgs-unstable, zed, home-manager, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config = {
          allowUnfree = true;
          allowUnfreePredicate = _: true;
        };
      };

      # secondary package set from unstable
      pkgsUnstable = import nixpkgs-unstable {
        inherit system;
        config = {
          allowUnfree = true;
          allowUnfreePredicate = _: true;
        };
      };
    in {
      # expose the Zed package from upstream as a flake package too
      packages.${system}.zed-latest = zed.packages.${system}.default;

      homeConfigurations."kofron@lifschitz" =
        home-manager.lib.homeManagerConfiguration {
          inherit pkgs;
          # pass unstable to HM modules as well
          extraSpecialArgs = {
            pkgsUnstable = pkgsUnstable;
            zedPkg = zed.packages.${system}.default;
          };
          modules = [
            ({ lib, ... }: {
              nixpkgs.overlays = [
                (final: prev: {
                  rofi-wayland = prev.rofi;
                })
              ];
            })
            ./home-manager/home.nix
            ({ lib, ... }: {
              home.stateVersion = lib.mkForce "25.05";
            })
          ];
        };
    };
}
