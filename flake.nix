{
  description = ''
    Depth-recorder

    This is a service that records depth data from a pressure sensor to a file.
  '';

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }: let
    systems = with flake-utils.lib.system; [
      x86_64-linux
      aarch64-linux
    ];
  in
    flake-utils.lib.eachSystem systems (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
      in {
        packages.default = nixpkgs.legacyPackages.${system}.callPackage ./default.nix {};
        formatter = nixpkgs.legacyPackages.${system}.alejandra;
      }
    )
    // {
      nixosModules.depth-recorder = import ./service.nix;
    };
}
