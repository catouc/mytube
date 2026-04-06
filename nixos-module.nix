{ config, lib, pkgs, ... }:
with lib;
  let cfg = config.mytube;
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
    users.groups."${cfg.group}" = {};
    users.users."${cfg.user}" = {
      isSystemUser = true;
      group = cfg.group;
    };

    systemd.services.mytube = {
      after = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      startAt = "*:0/30";
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/mytube -d ${cfg.databaseFile} update-channels 0"
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
}
