# Debian Setup Instructions

## Mirror Node

1. Download the Duniter .deb file.
2. Install the package: `dpkg -i duniter_vx.y.z.deb`.
3. Change the default configuration (at least the node name) by modifying `/etc/duniter/env_file`.
4. Start the service: `sudo systemctl start duniter-mirror.service`.
5. Enable the service at startup: `sudo systemctl enable duniter-mirror.service`.

## Smith Node

1. Download the Duniter .deb file.
2. Install the package: `dpkg -i duniter_vx.y.z.deb`.
3. Change the default configuration (at least the node name) by modifying `/etc/duniter/env_file`.
4. Create network keys using the same base path as in the config file: `duniter key generate-node-key --base-path <YOUR_BASE_PATH> --chain <YOUR_CHAIN>`.
5. Start the service: `sudo systemctl start duniter-validator.service`.
6. Enable the service at startup: `sudo systemctl enable duniter-validator.service`.

## Distance Oracle

A Smith node needs to be installed.

1. Change the default configuration by modifying `/etc/duniter/env_file`.
2. Start the service: `sudo systemctl start distance-oracle.timer`.
3. Enable the service at startup: `sudo systemctl enable distance-oracle.timer`.
