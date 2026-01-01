{
  pkgs,
  ...
}:
{
  # kal
  services.kal.enable = true;

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
        host.port = 8000;
        guest.port = 80;
      }
    ];
  };

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
