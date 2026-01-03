{
  pkgs,
  ...
}:
let
  radio = "wlan0";
  moduleName = "kal";
  network = "10.74.47";
  wifiPassword = "password";
in
{
  # kal
  services."${moduleName}".enable = true;

  # demo network
  networking = {
    firewall = {
      allowedTCPPorts = [
        53
      ];
      allowedUDPPorts = [
        67
        68
        69
      ];
      trustedInterfaces = [ radio ];
    };
    interfaces."${radio}" = {
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
      internalInterfaces = [ radio ];
      externalInterface = "eth0";
    };
  };
  services = {
    dnsmasq = {
      enable = true;
      settings = {
        dhcp-range = [ "${network}.10,${network}.100" ];
        server = [ "9.9.9.9" ];
        interface = radio;
        bind-interfaces = true;
        dhcp-authoritative = true;
        dhcp-option = [
          "option:router,${network}.1"
          "option:dns-server,${network}.1"
        ];
      };
    };
    hostapd = {
      enable = true;
      radios = {
        "${radio}" = {
          countryCode = "FR";
          channel = 1;
          networks."${radio}" = {
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
  };
  systemd.services = {
    dnsmasq.bindsTo = [ "network-addresses-${radio}.service" ];
    "network-addresses-${radio}".unitConfig.Restart = "always";
  };

  # keyboard layout
  console.useXkbConfig = true;
  services.xserver.xkb = {
    layout = "fr";
    variant = "ergol"; # remove this line for AZERTY default
  };

  # auth
  users.users.root.openssh.authorizedKeys.keys = [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFPWyZK9yJEyY7DqxN+A2h4+LccOoZGt2OdWEYvwzXzT nim@yupa"
  ];

  # base tools
  programs = {
    vim.enable = true;
    git.enable = true;
  };
  services = {
    openssh.enable = true;
  };

  # virtualisation
  hardware.firmware = [ pkgs.linux-firmware ];
  services.qemuGuest.enable = true;
  virtualisation = {
    cores = 2;
    memorySize = 8182;
    diskSize = 8192;
    graphics = true;
    qemu.options = [
      "-device"
      "virtio-vga"
      "-device"
      "qemu-xhci"
      "-device"
      "usb-host,vendorid=0x2357,productid=0x0138"
    ];
    spiceUSBRedirection.enable = true;
    forwardPorts = [
      {
        from = "host";
        host.port = 1883;
        guest.port = 1883;
      }
      {
        from = "host";
        host.port = 2222;
        guest.port = 22;
      }
      {
        from = "host";
        host.port = 7447;
        guest.port = 7447;
      }
      {
        from = "host";
        host.port = 8000;
        guest.port = 80;
      }
    ];
  };
  # grafana expect 3000, nginx proxy that to 80, and qemu formward to 8000
  services.grafana.settings.server.root_url = "http://localhost:8000";

  # nix things
  nix.settings = {
    experimental-features = [
      "nix-command"
      "flakes"
    ];
  };
  nixpkgs.hostPlatform = "x86_64-linux";
  system.stateVersion = "25.11";
}
