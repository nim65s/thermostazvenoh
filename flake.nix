{
  description = "Thermostat @ Azv w/ Zenoh";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    systems.url = "github:nix-systems/default";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } (
      { self, lib, ... }:
      {
        systems = import inputs.systems;
        flake = {
          nixosModules.default = ./module.nix;
          nixosConfigurations.vm = inputs.nixpkgs.lib.nixosSystem {
            modules = [
              "${inputs.nixpkgs}/nixos/modules/virtualisation/qemu-vm.nix"
              self.nixosModules.default
              ./vm.nix
            ];
          };
        };
        perSystem =
          {
            pkgs,
            system,
            ...
          }:
          {
            _module.args.pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [ (import inputs.rust-overlay) ];
            };
            devShells.default =
              with pkgs;
              mkShell {
                packages = [
                  (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
                  espflash
                  esp-generate
                  probe-rs-tools
                ];
              };
          };
      }
    );
}
