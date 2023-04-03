# How to become a (black)smith

## Publish a node

### Duniter part

See [docker documentation](../../docker/README.md) to install, configure, and start a node. For a smith node, you want to set `DUNITER_VALIDATOR` to `true`.

### Reverse proxy part

See [nginx reverse proxy](./nginx_reverse_proxy.md).

## Join the Smith WoT

Only members of the smith WoT can author blocks. This WoT is a subset of the main WoT, hence before joining it you need to join the main WoT.

1. Create an SSH bridge from your computer to your server: `ssh -L 9945:localhost:9945 SSH_USER@YOUR_SERVER`
2. Install PolkadotJS browser extension. It will manage your private keys and known pubkeys safely.
3. Go to [a PolkadotJS web UI](https://polkadot.js.org/apps/?rpc=ws://127.0.0.1/ws:9945). (it's not the same thing as the browser extension)
  - If using another port or address, change it accordingly in the left panel.
4. In the UI: developer > RPC call > author > rotateKeys() and copy the result in clipboard
5. In the UI: developer > extrinsics > YOUR_SMITH_ACCOUNT > smithsMembership > requestMembership(metadata)
  - add your p2p endpoint (optional)
  - add your session keys
  - send the query
6. Wait 48h to ensure your node keeps sync (**both** best **and** finalized block numbers must increase every 6s)
7. Await at least 3 smith certifications. Members of the smith WoT can certify you with this extrinsic:
  - In the UI: developer > extrinsics > CERTIFIER_SMITH_ACCOUNT > smithsCert > addCert(receiver)
  - This is not automatic, you can ask for certs on the forum or the Matrix chatroom.
8. In the UI: developer > extrinsics > YOUR_SMITH_ACCOUNT > smithsMembership > claimMembership(maybe_idty_id)
  - maybe_idty_id can be left empty since your identity id will be infered from your account address.

All extrinsics can be sent while connected to any Duniter node, but the RPC calls need a direct connection to your server. As some RPC calls should not be publicly callable for security reasons, the only ways to call them is from the server localhost or using an SSH bridge or other kind of secure tunnel.

rotateKeys can be called anytime you want. Then you have to call setSessionKeys with the new keys.

## Validate blocks (blacksmith work)

Once all the previous steps are completed, you can start to actually author blocks.

1. In the UI: developer > extrinsics > YOUR_SMITH_ACCOUNT > authorityMembers > goOnline()
2. Every less than 2 months, make the RPC call author.rotateKeys then the extrinsic authorityMembers.setSessionKeys.

If you're not able to monitor, reboot, act on your node, goOffline() to avoid penalty to the blockchain and to you. It will take effect after 2h, so please do it in advance if you plan to disconnect your server soon. goOnline can always be called after this, but after 100 days without authoring blocks you will loose your smith membership.

## Upgrade your node with minimal interruption

1. Modify docker image tag on your compose file
2. Run `docker compose pull`, this will pull the new image.
3. Run `docker compose up -d --remove-orphans`, this will recreate the container
4. Verify that your node restarted well `docker compose logs duniter-validator`
5. Remove the old image `docker rmi duniter/duniter-v2s:OLD_TAG`
