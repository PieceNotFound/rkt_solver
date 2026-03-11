{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rust = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };

        cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      rec {
        devShell = pkgs.mkShell rec {
          buildInputs =
            (with pkgs; [
              sccache
              rust-analyzer
              rust
              pkg-config
            ]);

          RUST_BACKTRACE = 1;
          RUSTC_WRAPPER = "sccache";
          SCCACHE_SERVER_PORT = "54226";
          RUSTFLAGS = "-C target-cpu=native";

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

          shellHook = ''
            export PATH=$PATH:~/.cargo/bin
          '';
        };

        packages.rkt-solver = pkgs.rustPlatform.buildRustPackage {
          pname = cargo.package.name;
          version = cargo.package.version;

          src = ./.;

          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          meta = with pkgs.lib; {
            homepage = cargo.package.homepage;
            description = cargo.package.description;
            # TODO: Change
            license = licenses.mit;
          };
        };

        defaultPackage = packages.rkt-solver;
      }
    );
}
