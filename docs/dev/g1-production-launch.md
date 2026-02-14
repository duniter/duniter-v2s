# Mise en production du réseau Ğ1 v2

Guide opérationnel pour le lancement du réseau Ğ1 sur Duniter-v2S.

## Prérequis techniques

### Machine de build

- Linux (Ubuntu 22.04), 16 Go RAM, 8 CPU, 50 Go disque
- Docker ou Podman (`podman machine set --memory 16384 --cpus 8`)
- Rust (toolchain nightly-2025-08-07 via `rust-toolchain.toml`)
- Outils : `cmake pkg-config libssl-dev git build-essential clang protobuf-compiler jq`

### Credentials

```bash
export GITLAB_TOKEN="..."        # GitLab Access Token (scope: api)
export DUNITERTEAM_PASSWD="..."  # Docker Hub org duniter
```

### Serveurs

- **Bootstrap** : 4 Go RAM, 2 CPU, 100 Go SSD, ports 30333 (P2P) + 9944 (RPC local)
- **Forgerons** : mêmes specs, min 3-5 au lancement
- **Miroirs** : port 443 (reverse proxy WSS vers 9944) + 30333

---

## Lacunes identifiées (à corriger avant le jour J)

> Ces 4 points **bloquent** le lancement g1 en l'état actuel du code.

### 1. `g1_live` absent de `command.rs`

`xtask/src/network/build_network_specs.rs:69` utilise `--chain g1_live`, mais `node/src/command.rs` ne gère pas ce chain ID (contrairement à `gtest_live` ligne 145 et `gdev_live` ligne 107).

**Fix** : Ajouter dans `command.rs` un bloc `"g1_live" => { ... }` calqué sur `gtest_live`, et dans `chain_spec/g1.rs` les fonctions `live_chainspecs()` et la struct `ClientSpec` (calquées sur `gtest.rs`).

### 2. `resources/g1.yaml` manquant

Pas de fichier de configuration réseau g1 (forgerons, sudo, comité technique). Seuls `gdev.yaml` et `gtest.yaml` existent.

### 3. `node/specs/g1_client-specs.yaml` manquant

Pas de client specs g1 (bootnodes, télémétrie, token symbol).

### 4. `chain_spec/g1.rs` incomplet

Manquent `ClientSpec`, `live_chainspecs()`, `development_chainspecs()` (présentes dans `gtest.rs`).

---

## Procédure de lancement

### Étape 1 — Branche réseau

```bash
git checkout master && git pull
git checkout -b network/g1-1000
rm -rf release/*
```

Vérifier `spec_version: 1000` dans `runtime/g1/src/lib.rs` et la version dans `node/Cargo.toml`.

### Étape 2 — Données de migration Ğ1 v1

```bash
cargo xtask release network g1-data
# ou avec un dump spécifique :
cargo xtask release network g1-data --dump-url "https://dl.cgeek.fr/public/auto-backup-g1-duniter-1.8.7_YYYY-MM-DD_00-00.tgz"
```

Utilise Docker avec `py-g1-migrator` pour extraire identités, soldes, certifications depuis le dump LevelDB de Duniter v1.

Fichiers générés dans `release/network/` : `genesis.json`, `block_hist.json`, `cert_hist.json`, `tx_hist.json`.

```bash
# Vérification
jq '.identities | length' release/network/genesis.json
jq '.initial_monetary_mass' release/network/genesis.json
```

### Étape 3 — Fichiers de configuration réseau

Créer `resources/g1.yaml` (modèle : `resources/gtest.yaml`) :

```yaml
ud: 1148                          # DU initial en centièmes de Ğ1
first_ud: null
first_ud_reeval: 1766232000000    # ms depuis epoch, à ajuster

clique_smiths:
  - name: "forgeron1"
  - name: "forgeron2"
  - name: "forgeron3"
  - name: "forgeron_bootstrap"
    session_keys: "0x..."         # généré à l'étape 4.1

sudo_key: "5..."
treasury_funder_pubkey: "..."
technical_committee: ["forgeron1", "forgeron2", "forgeron3", "forgeron_bootstrap"]
```

Créer `node/specs/g1_client-specs.yaml` (modèle : `node/specs/gtest_client-specs.yaml`) :

```yaml
name: "Ğ1"
id: "g1"
chainType: "Live"
protocolId: "g1"
bootNodes:
  - "/dns/g1-boot1.duniter.org/tcp/30333/p2p/<PEER_ID>"
telemetryEndpoints:
  - ["/dns/telemetry.polkadot.io/tcp/443/x-parity-wss/%2Fsubmit%2F", 0]
properties:
  tokenDecimals: 2
  tokenSymbol: "Ğ"
```

### Étape 4 — Génération des clés de session bootstrap

```bash
docker run --rm duniter/duniter-v2s-g1-1000:latest -- key generate
# → noter la phrase secrète + clé publique SS58

docker run --rm duniter/duniter-v2s-g1-1000:latest -- \
  key generate-session-keys --chain g1_local --suri "<phrase secrète>"
# → coller le résultat hex dans resources/g1.yaml, champ session_keys
```

### Étape 5 — Build du runtime WASM

```bash
cargo xtask release network build-runtime g1
cp release/g1_runtime.compact.compressed.wasm release/network/
```

Utilise `srtool` (Docker `paritytech/srtool:1.88.0`) pour un build reproductible. Le hash SHA256 est dans `release/srtool_output_g1.json`.

### Étape 6 — Génération des specs réseau

```bash
cargo xtask release network build-specs g1
```

Exécute en interne :
```bash
DUNITER_GENESIS_DATA=release/network/genesis.json \
WASM_FILE=release/network/g1_runtime.compact.compressed.wasm \
cargo run --release --features g1 --no-default-features build-spec --chain g1_live
```

Résultat : `release/network/g1.json`

### Étape 7 — Release réseau GitLab

```bash
cargo xtask release network create g1-1000 network/g1-1000
```

Upload sur GitLab : genesis.json, g1.json, g1.yaml, WASM, fichiers Squid.

### Étape 8 — Release client

```bash
cargo xtask release client build-raw-specs g1-1000
cargo xtask release client create g1-1000 network/g1-1000
cargo xtask release client trigger-builds g1-1000 network/g1-1000
```

La dernière commande déclenche la CI GitLab (DEB/RPM x64+ARM, Docker amd64+arm64, manifest multi-arch).

Image Docker résultante : `duniter/duniter-v2s-g1-1000:1000-<client_version>`

### Étape 9 — Déploiement du nœud bootstrap

```yaml
# docker-compose.yml sur le serveur bootstrap
services:
  duniter-g1-smith:
    image: duniter/duniter-v2s-g1-1000:1000-0.12.0
    restart: unless-stopped
    ports:
      - 127.0.0.1:9944:9944   # RPC local uniquement !
      - 30333:30333
    environment:
      DUNITER_NODE_NAME: g1-bootstrap
      DUNITER_CHAIN_NAME: g1
      DUNITER_VALIDATOR: "true"
      DUNITER_PRUNING_PROFILE: light
      DUNITER_PUBLIC_ADDR: /dns/g1-boot1.duniter.org/tcp/30333
      DUNITER_LISTEN_ADDR: /ip4/0.0.0.0/tcp/30333
    volumes:
      - g1-data:/var/lib/duniter

  distance-oracle:
    image: duniter/duniter-v2s-g1-1000:1000-0.12.0
    restart: unless-stopped
    entrypoint: docker-distance-entrypoint
    environment:
      ORACLE_RPC_URL: ws://duniter-g1-smith:9944
      ORACLE_RESULT_DIR: /var/lib/duniter/chains/g1/distance/
      ORACLE_EXECUTION_INTERVAL: 1800
    volumes:
      - g1-data:/var/lib/duniter

volumes:
  g1-data:
```

```bash
docker compose up -d
docker compose logs duniter-g1-smith | grep "Local node identity"
# → noter le Peer ID (12D3KooW...) pour g1_client-specs.yaml
```

### Étape 10 — Rotation des clés de session

Les clés du genesis viennent de la machine de build. Les roter sur le serveur :

```bash
curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"author_rotateKeys","params":[]}' \
  http://127.0.0.1:9944
```

Puis soumettre on-chain via `session.setKeys` (polkadot.js/apps ou subxt). Prise d'effet après une epoch (4h).

### Étape 11 — Forgerons additionnels

Chaque forgeron : même docker-compose adapté (nom, adresse publique) + bootnode du bootstrap. Puis :

```bash
# Générer les clés de session
curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"author_rotateKeys","params":[]}' \
  http://127.0.0.1:9944
# Soumettre via session.setKeys
```

Alternative Debian : `dpkg -i duniter.deb` + configurer `/etc/duniter/env_file` + `systemctl start duniter-smith distance-oracle`.

### Étape 12 — Nœuds miroirs

```yaml
services:
  duniter-g1-mirror:
    image: duniter/duniter-v2s-g1-1000:1000-0.12.0
    ports: ["9944:9944", "30333:30333"]
    environment:
      DUNITER_NODE_NAME: g1-mirror
      DUNITER_CHAIN_NAME: g1
      DUNITER_PRUNING_PROFILE: archive
      DUNITER_PUBLIC_RPC: wss://rpc.g1.duniter.org
    volumes: [g1-mirror:/var/lib/duniter]
```

Reverse proxy nginx pour WSS :

```nginx
server {
    listen 443 ssl;
    server_name rpc.g1.duniter.org;
    location / {
        proxy_pass http://127.0.0.1:9944;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }
}
```

---

## Vérifications post-lancement

```bash
# Blocs produits (toutes les 6s)
curl -s -d '{"id":1,"jsonrpc":"2.0","method":"chain_getHeader","params":[]}' \
  -H "Content-Type: application/json" http://127.0.0.1:9944 | jq '.result.number'

# Finalisation GRANDPA (nécessite 2/3 des validateurs)
curl -s -d '{"id":1,"jsonrpc":"2.0","method":"chain_getFinalizedHead","params":[]}' \
  -H "Content-Type: application/json" http://127.0.0.1:9944
```

Checklist :
- [ ] Blocs produits régulièrement
- [ ] Finalisation active
- [ ] Forgerons connectés et en ligne
- [ ] Oracle de distance opérationnel
- [ ] Identités et certifications migrées
- [ ] Masse monétaire correcte
- [ ] DU créé après 24h
- [ ] RPC public accessible
- [ ] Télémétrie visible

---

## Paramètres runtime Ğ1

Source : `runtime/g1/src/parameters.rs` — paramètres **figés** (non modifiables sans upgrade runtime).

| Paramètre | Valeur |
|-----------|--------|
| Bloc | 6s |
| Epoch BABE | 4h (14 400 blocs) |
| Max validateurs | 100 |
| SS58 Prefix | 4450 |
| Existential deposit | 2,00 Ğ |
| DU création | 24h |
| DU réévaluation | ~6 mois |
| Cert min pour membre | 5 |
| Cert période | 5 jours |
| Cert validité | 2 ans |
| Adhésion | 1 an |
| Renouvellement adhésion | 2 mois |
| Création identité | 1/mois |
| Changement clé | 1/6 mois |
| Smith cert min | 3 |
| Smith max certifs émises | 12 |
| Smith inactivité max | 48 blocs (~4.8 min) |
| Distance max | 5 pas |
| Référents accessibles min | 80% |
