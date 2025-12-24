{
  config,
  lib,
  pkgs,
  ...
}:
let
  moduleName = "thermostazvenoh";
  cfg = config.services."${moduleName}";
in
{
  options = {
    enable = lib.mkEnableOption "${moduleName} service";
    radio = lib.mkOption {
      type = lib.types.str;
      description = "hostapd interface";
    };
  };
  config = lib.mkIf cfg.enable {
    services = {
      grafana = {
        enable = true;
        provision = {
          enable = true;
          # dashboards.settings.providers = [
          #   {
          #     options.path = ./dashboards;
          #   }
          # ];
          datasources.settings.datasources = [
          ];
        };
      };
      hostapd = {
        enable = true;
        radio = {
          "${cfg.radio}" = {
            inherit (cfg) countryCode;
            networks."${cfg.radio}" = {
              SSID = moduleName;
              authentication.saePasswords = [ { password = moduleName; } ];
            };
          };
        };
      };
      zenohd = {
        enable = true;
        plugins = [ pkgs.zenoh-plugin-mqtt ];
        backends = [ ];
        settings.plugins = {
          mqtt = { };
          storage_manager.storages."${moduleName}" = {
            key_expr = "tele/**";
            volume.id = "memory";
          };
        };
      };
    };
    networking.firewall.allowedTCPPorts = [ 7447 ];
  };
}
