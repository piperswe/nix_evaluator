{
  inputs = {
    mkRustFlake.url = github:piperswe/mkrustflake;

    flake-compat = {
      url = github:edolstra/flake-compat;
      flake = false;
    };
  };

  outputs = { self, mkRustFlake, ... }: mkRustFlake.lib.mkRustFlake {
    name = "nix_evaluator";
    version = "0.0.0";
    src = ./.;
    cargoLock = ./Cargo.lock;
  };
}
