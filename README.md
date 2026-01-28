# Kal

An over-engineered home-automation setup to play with rust, zenoh, nix, embassy, esp32, grafana, influxdb, mqtt, and home-assistant.

## example VM

```
nix run .#nixosConfigurations.vm.config.system.build.vm
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 2222 root@localhost
```

## embed

```
cd kal-embed
cargo run
```

## Real setup

The architecture I needed for this demo is not exactly the same as my real setup, which is very simplified, and available here:
https://github.com/nim65s/dotfiles/tree/main/machines/calcifer/kal
