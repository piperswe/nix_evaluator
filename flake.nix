{
  inputs = {
    nixpkgs.url = github:nixos/nixpkgs;
    flake-utils.url = github:numtide/flake-utils;
    rust-overlay.url = github:oxalica/rust-overlay;

    flake-compat = {
      url = github:edolstra/flake-compat;
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlay
        ];
      };
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "clippy" ];
      };
    in
    {
      packages.nix_evaluator = pkgs.rustPlatform.buildRustPackage {
        pname = "nix_evaluator";
        version = "0.0.0";
        nativeBuildInputs = [
          rust
        ];

        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };
      defaultPackage = self.packages.${system}.nix_evaluator;

      devShell = self.defaultPackage.${system};

      hydraJobs = {
        packages = self.packages.${system};
      };
    });
}
