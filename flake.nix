{
  description = "Home Manager configuration for kofron";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ nixpkgs, home-manager, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config = {
          allowUnfree = true;
          allowUnfreePredicate = _: true;
        };
      };
    in {
      homeConfigurations."kofron@lifschitz" =
        home-manager.lib.homeManagerConfiguration {
          inherit pkgs;
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
              home.stateVersion = lib.mkForce "24.05";
            })
          ];
        };
    };
}
