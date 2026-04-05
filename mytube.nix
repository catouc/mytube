{
  stdenv,
  lib,
  rustPlatform,
  sqlite,
}:

rustPlatform.buildRustPackage {
  pname = "mytube";
  version = "v0.6.0";
  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.difference ./. ./result;
  };
  propagatedBuildInputs = [
		sqlite
	];
  cargoHash = "sha256-ccTdpeMMfuZ7SQADk36SAmeKM9CmkGHgWiHhe5JWA+E=";
	cargoLock.lockFile = ./Cargo.lock;
}
