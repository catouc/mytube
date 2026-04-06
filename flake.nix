{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  description = "";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";     
      pkgs = import nixpkgs { inherit system; };
    in
    {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "mytube";
        version = "v0.6.0";
        src = ./.;
        propagatedBuildInputs = [
          pkgs.sqlite
        ];
        cargoHash = "sha256-ccTdpeMMfuZ7SQADk36SAmeKM9CmkGHgWiHhe5JWA+E=";
        cargoLock.lockFile = ./Cargo.lock;
      };
        
      nixosModules = {
        default = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.mytube;
        in
        {
          options.mytube = {
            enable = mkEnableOption "Enable the mytube background services";

            user = mkOption {
              type = types.str;
              default = "mytube";
              example = "mytube";
              description = "The user to run this service under";
            };

            package = mkOption {
              type = types.package;
              default = pkgs.mytube;
              description = "Overwrite the package to use";
            };

            group = mkOption {
              type = types.str;
              default = "mytube";
              example = "mytube";
              description = "The group to run this service under";
            };

            databaseFile = mkOption {
              type = types.path;
              default = "/var/mytube/mytube.db";
              example = "/home/user/mytube.db";
              description = "The sqlite database file location";
            };
          };

          config = mkIf cfg.enable {
            systemd.services.mytube = {
              after = [ "network-online.target" ];
              wantedBy = [ "multi-user.target" ];
              startAt = "*:0/30";
              serviceConfig = {
                ExecStart = "${cfg.package}/bin/mytube update-channels 0";
                User = cfg.user;
                Group = cfg.group;

                CapabilityBoundingSet="";
                LockPersonality="yes";
                NoNewPrivileges = true;
                PrivateDevices = "yes";
                PrivateTmp = "yes";
                PrivateUsers="yes";
                ProcSubset="pid";
                ProtectClock="yes";
                ProtectControlGroups = "strict";
                ProtectHome="yes";
                ProtectHostname="yes";
                ProtectKernelLogs="yes";
                ProtectKernelModules = "yes";
                ProtectKernelTunables = "yes";
                ProtectProc="invisible";
                ProtectSystem = "strict";
                RemoveIPC="yes";
                RestrictNamespaces="yes";
                RestrictRealtime="yes";
                RestrictSUIDSGID = "yes";
                SystemCallErrorNumber="EPERM";
                SystemCallFilter="@system-service";
                SystemCallArchitectures="native";
              }; 
            };
          };
        };
      };

      devShells.x86_64-linux.default = pkgs.mkShell {
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
