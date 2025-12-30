# Thermostat @ Home, edition zenoh-nostd

Zenoh, but keep MQTT compat and tasmota formats with `cmnd` & `tele`

## example use case

between 5 and 8 am, if less that 15.5°C, turn the heater on until 16.3°C

## example VM
```
nix run .#nixosConfigurations.vm.config.system.build.vm
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 2222 root@localhost
```


