{
  config,
  lib,
  pkgs,
  ...
}:
let
  moduleName = "kal";
  cfg = config.services."${moduleName}";
  network = "10.74.47";
  secretInfluxDBToken = "please-use-sops-nix-or-agenix";
  wifiPassword = "password";
in
{
  options = {
    services."${moduleName}" = {
      enable = lib.mkEnableOption "${moduleName} service";
      countryCode = lib.mkOption {
        type = lib.types.str;
        default = "FR";
        description = "hostapd country code";
      };
      radio = lib.mkOption {
        type = lib.types.str;
        default = "wlan0";
        description = "hostapd interface";
      };
    };
  };
  config = lib.mkIf cfg.enable {
    services = {
      dnsmasq = {
        enable = true;
        settings = {
          dhcp-range = [ "${network}.10,${network}.100" ];
          server = [ "9.9.9.9" ];
          interface = cfg.radio;
          bind-interfaces = true;
          dhcp-authoritative = true;
          dhcp-option = [
            "option:router,${network}.1"
            "option:dns-server,${network}.1"
          ];
        };
      };
      grafana = {
        enable = true;
        provision = {
          enable = true;
          dashboards.settings.providers = [
            {
              options.path = ./dashboards;
            }
          ];
          datasources.settings.datasources = [
            {
              name = "InfluxDB";
              type = "influxdb";
              isDefault = true;
              access = "proxy";
              url = "http://localhost:8086";
              jsonData = {
                version = "Flux";
                organization = moduleName;
                defaultBucket = moduleName;
              };
              secureJsonData.token = secretInfluxDBToken;
            }
          ];
        };
      };
      hostapd = {
        enable = true;
        radios = {
          "${cfg.radio}" = {
            inherit (cfg) countryCode;
            channel = 1;
            networks."${cfg.radio}" = {
              ssid = moduleName;
              authentication = {
                # wpa3 does not seem to work with embassy esp yet
                mode = "wpa2-sha256";
                wpaPassword = wifiPassword;
              };
            };
          };
        };
      };
      influxdb2 = {
        enable = true;
        provision = {
          enable = true;
          initialSetup = {
            bucket = moduleName;
            organization = moduleName;
            tokenFile = pkgs.writeText "token" secretInfluxDBToken;
            passwordFile = pkgs.writeText "password" "not-${secretInfluxDBToken}";
          };
          # organizations = {
          #   "${moduleName}" = {
          #     description = "${moduleName} organization";
          #     auths."${moduleName}" = {
          #       writeBuckets = [ "${moduleName}" ];
          #       tokenFile = pkgs.writeText "token" secretInfluxDBToken;
          #     };
          #   };
          # };
        };
      };
      nginx = {
        enable = true;
        virtualHosts.localhost = {
          default = true;
          locations."/" = {
            recommendedProxySettings = true;
            proxyWebsockets = true;
            proxyPass =
              let
                grafana = config.services.grafana.settings.server;
              in
              "http://${grafana.http_addr}:${toString grafana.http_port}";
          };
        };
      };
      zenohd = {
        enable = true;
        plugins = [ pkgs.zenoh-plugin-mqtt ];
        backends = [ pkgs.zenoh-backend-influxdb ];
        settings.plugins = {
          mqtt = { };
          rest.http_port = 8000;
          storage_manager = {
            volumes.influxdb2 = {
              url = "http://localhost:8086";
              private = {
                org_id = moduleName;
                token = secretInfluxDBToken;
              };
            };
            storages."${moduleName}" = {
              key_expr = "kal/**";
              volume = {
                id = "influxdb2";
                db = moduleName;
                private = {
                  org_id = moduleName;
                  token = secretInfluxDBToken;
                };
              };
            };
          };
        };
      };
    };
    networking = {
      firewall = {
        allowedTCPPorts = [
          53
          80
          1883
          7447
        ];
        allowedUDPPorts = [
          67
          68
          69
        ];
        trustedInterfaces = [ cfg.radio ];
      };
      interfaces."${cfg.radio}" = {
        ipv4.addresses = [
          {
            address = "${network}.1";
            prefixLength = 24;
          }
        ];
        useDHCP = false;
      };
      nat = {
        enable = true;
        internalInterfaces = [ "wlan0" ];
        externalInterface = "eth0";
      };
      networkmanager = {
        unmanaged = [ "interface-name:${cfg.radio}" ];
        wifi.powersave = false;
      };
    };
    systemd.services.zenohd.after = [ "influxdb2.service" ];
  };
}
