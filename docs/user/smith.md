# How to become a (black)smith

## Publish a node

### Duniter part

See [docker documentation](../../docker/README.md) to install, configure, and start a node. For a smith node, you want to set `DUNITER_VALIDATOR` to `true`.

### Reverse proxy part

See [nginx reverse proxy](./nginx_reverse_proxy.md).

## Join the Smith WoT

- add polkadot webextension to be able to authentificate with your account.
- Go to [any node with polkadotjs ui](https://gdev.1000i100.fr/dev-ui/?rpc=wss://gdev.1000i100.fr/ws)
- Ask to join Smith WoT (you need to already be in the main WoT)
  - developer > extrinsics > YOUR_SMITH_ACCOUNT > smithMembership > requestMemberShip(metadata)
  - add your p2p endpoint (optional)
  - add your session key (follow point 1 to 4 from Validate blocks > Generate and publish your session key)
  - Send the query
- Await smith certification : developer > extrinsics > CERTIFIER_SMITH_ACCOUNT > smithCert > addCert(receiver)

When you have at least 3 certifications, your'in!

## Validate blocks (blacksmith work)

- Generate and publish your session keys
  1. create an ssh bridge from your desktop/laptop to your server : `ssh -L 9945:localhost:9945 SSH_USER@YOUR_SERVER`
  2. In your browser go to [polkadotjs : ws://localhost:9945](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2Flocalhost%3A9945#/explorer)
  3. In the UI : developer > appel RPC > author > rotateKey() and copy the result in clipboard
  4. In the UI : developer > extrinsics > YOUR_SMITH_ACCOUNT > authorityMembers > setSessionKeys(keys) then paste your session keys and run the query.
  5. In the UI : developer > extrinsics > YOUR_SMITH_ACCOUNT > authorityMembers > claimMembership(keys)
  6. **wait 48h to verify you keep sync**
- Join
  - In the UI : developer > extrinsics > YOUR_SMITH_ACCOUNT > authorityMembers > goOnline()

If you're not able to monitor, reboot, act on your node, goOffline() to avoid penality to the blockchain and to you.

## Upgrade your node with minimal interruption

1. Modify docker image tag on your compose file
2. Run `docker compose pull`, this will pull the new image.
3. Run `docker compose up -d --remove-orphans`, this will recreate the container
4. Verify that your node restarted well `docker compose logs duniter-validator`
5. Remove the old image `docker rmi duniter/duniter-v2s:OLD_TAG`
