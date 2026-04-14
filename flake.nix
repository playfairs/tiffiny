{
  description = "tiffiny - Convert Audio files into Images using TIFF Headers and RAW Data Manipulation.";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
  let
    forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
  in
  {
    devShells = forAllSystems (system: let
      pkgs = import nixpkgs { inherit system; };
    in {
      default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          cargo
          rustfmt
          clippy
        ];
      };
    });

    packages = forAllSystems (system: let
      pkgs = import nixpkgs { inherit system; };
    in rec {
      tiffiny = pkgs.rustPlatform.buildRustPackage {
        pname = "tiffiny";
        version = "0.1.0";

        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };
      };
      default = tiffiny;
    });
  };
}
