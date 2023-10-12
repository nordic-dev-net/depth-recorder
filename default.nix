{
  rustPlatform,
  pkg-config,
  pkgs,
}:
rustPlatform.buildRustPackage {
  pname = "depth-recorder";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };
}
