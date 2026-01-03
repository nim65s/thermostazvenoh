{
  lib,
  rustPlatform,
}:
let
  cargo = lib.importTOML ./kal-daemon/Cargo.toml;
in
rustPlatform.buildRustPackage {
  inherit (cargo.package) name version;
  src = lib.cleanSource ./kal-daemon;
  cargoLock = {
    lockFile = ./kal-daemon/Cargo.lock;
  };
  meta = {
    description = "schedule heater activation";
    mainProgram = cargo.package.name;
  };
}
