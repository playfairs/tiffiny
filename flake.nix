{
  description = "";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { ... }@inputs:
    let
      inherit (inputs) fenix nixpkgs;

      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    in
    {
      formatter = forAllSystems (
        system:
        import ./nix/formatter.nix {
          inherit inputs;
          pkgs = import nixpkgs { inherit system; };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };

          toolchain =
            with fenix.packages.${pkgs.stdenv.system};
            combine [
              latest.toolchain
              targets.wasm32-unknown-unknown.latest.rust-std
            ];
        in
        {
          default = pkgs.mkShell {
            LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";
            CC = "${pkgs.llvmPackages_18.clang}/bin/clang";
            CXX = "${pkgs.llvmPackages_18.clang}/bin/clang++";

            buildInputs = with pkgs; [
              toolchain
              pkg-config
              openssl
              ffmpeg_7
              opencv
              llvmPackages_18.libclang
              llvmPackages_18.clang
            ];
          };
        }
      );

      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        rec {
          tiffiny = pkgs.rustPlatform.buildRustPackage {
            pname = "tiffiny";
            version = "0.1.0";

            LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";
            CC = "${pkgs.llvmPackages_18.clang}/bin/clang";
            CXX = "${pkgs.llvmPackages_18.clang}/bin/clang++";

            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = with pkgs; [
              openssl
              ffmpeg_7
              opencv
              llvmPackages_18.libclang
              llvmPackages_18.clang
            ];
          };

          default = tiffiny;
        }
      );
    };
}
