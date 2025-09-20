{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils/v1.0.0";
  };

  description = "";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        build = pkgs.rustPlatform.buildRustPackage {
          pname = "replace";
          version = "v0.2.0";
          src = ./.;
	  propagatedBuildInputs = [
		pkgs.sqlite
	  ];
          cargoHash = "sha256-ccTdpeMMfuZ7SQADk36SAmeKM9CmkGHgWiHhe5JWA+E=";
	  cargoLock.lockFile = ./Cargo.lock;
        };
      in
      rec {
        packages = {
          replace = build;
          default = build;
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
	      litecli
	      nixd
	      openssl
	      pkg-config
	      sqlite
	      yt-dlp
              cargo
              rust-analyzer
              rustPackages.clippy
              rustPackages.rustfmt
              rustc
            ];
          };
        };
      }
    );
}

