# Mise en production du réseau Ğ1 v2

Guide opérationnel pour le lancement du réseau Ğ1 sur Duniter-v2S.

## Prérequis techniques

### Machine de build

- Linux (Ubuntu 22.04), 16 Go RAM, 8 CPU, 50 Go disque
- Docker ou Podman (`podman machine set --memory 16384 --cpus 8`)
- Rust (toolchain 1.88.0 via `rust-toolchain.toml`)
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

## Procédure de lancement

> **Prérequis :** Les actions de la checklist `g1-launch-checklist.md` (section « En amont du jour J ») doivent être réalisées.

### Étape 1 — Branche réseau

```bash
git checkout master && git pull
git checkout -b network/g1-1000
rm -rf release/*
```

Vérifier `spec_version: 1000` dans `runtime/g1/src/lib.rs` et vérifier que la **version client** dans `node/Cargo.toml` a bien été bumpée (checklist A6, ex : `version = "1.0.0"`). Cette version client est distincte du `spec_version` runtime : elle identifie le binaire du nœud et apparaît dans le tag Docker (`1000-<client_version>`) et le nom de la release GitLab.

Vérifier que les fichiers modifiés par la checklist (section « En amont du jour J ») sont bien présents :

- `resources/g1.yaml` (clique_smiths, technical_committee, paramètres économiques)
- `node/specs/g1_client-specs.yaml` (bootNodes avec les Peer ID de l'étape A3)
- `node/Cargo.toml` (version client bumpée en checklist A6)

```bash
# Committer les changements et pousser la branche
git add resources/g1.yaml node/specs/g1_client-specs.yaml runtime/g1/src/lib.rs node/Cargo.toml
git commit -m "chore(g1): configure network/g1-1000"
git push -u origin network/g1-1000
```

> **Important :** La branche doit exister sur GitLab **avant** l'étape 5, car la release réseau crée un tag à partir de cette branche. La CI déclenchée à l'étape 6 compilera également le code depuis cette branche.

### Étape 2 — Données de migration Ğ1 v1

```bash
cargo xtask release network g1-data
```

```bash
# Vérification
jq '.identities | length' release/network/genesis.json
jq '.initial_monetary_mass' release/network/genesis.json
```

<details><summary>Options et fichiers produits</summary>

Dump spécifique : `cargo xtask release network g1-data --dump-url "https://...tgz"`

Fichiers générés dans `release/network/` : `genesis.json`, `block_hist.json`, `cert_hist.json`, `tx_hist.json`.
</details>

### Étape 3 — Build du runtime WASM

```bash
cargo xtask release network build-runtime g1
```

> **Note ARM :** L'image srtool est amd64 uniquement. Sur Mac ARM, allouez 16 Go+ à Docker Desktop ou utilisez une machine x86_64.

<details><summary>Fonctionnement</summary>

Build reproductible via `srtool` (Docker `paritytech/srtool:1.88.0`). Le WASM est généré dans `release/network/` et le hash SHA256 dans `release/network/network_srtool_output.json`.
</details>

### Étape 4 — Génération des specs réseau

```bash
cargo xtask release network build-specs g1
```

<details><summary>Fonctionnement</summary>

Exécute en interne :
```bash
DUNITER_GENESIS_DATA=release/network/genesis.json \
WASM_FILE=release/network/g1_runtime.compact.compressed.wasm \
cargo run --release --features g1 --no-default-features build-spec --chain g1_live
```
Résultat : `release/network/g1.json`
</details>

### Étape 5 — Release réseau GitLab

```bash
cargo xtask release network create g1-1000 network/g1-1000
```

<details><summary>Assets uploadés</summary>

genesis.json, g1.json, g1.yaml, WASM, block_hist.json.gz, cert_hist.json.gz, tx_hist.json.gz.
</details>

### Étape 6 — Release client

Créer le jalon GitLab **avant** de lancer la release :

1. Ouvrir https://git.duniter.org/nodes/rust/duniter-v2s/-/milestones/new
2. Titre : `client-<version>` (ex : `client-1.0.0`, la version est dans `node/Cargo.toml`)
3. Cliquer "Create milestone"

```bash
cargo xtask release client build-raw-specs g1-1000
cargo xtask release client create g1-1000 network/g1-1000
cargo xtask release client trigger-builds g1-1000 network/g1-1000
```

Image Docker résultante : `duniter/duniter-v2s-g1-1000:1000-<client_version>`

<details><summary>Rôle de chaque commande</summary>

- `build-raw-specs` : génère `g1-raw.json` localement (gitignored).
- `create` : upload les specs vers la release GitLab.
- `trigger-builds` : passe l'URL de `g1-raw.json` aux jobs CI via `RAW_SPEC_URL`, déclenche DEB/RPM x64+ARM, Docker amd64+arm64, manifest multi-arch.
</details>

### Étape 7 — Déploiement du noeud bootstrap

```yaml
# docker-compose.yml sur le serveur bootstrap
services:
  duniter-g1-smith:
    image: duniter/duniter-v2s-g1-1000:1000-1.0.0
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
      - ./node.key:/var/lib/duniter/node.key:ro  # clé réseau générée en A3

  distance-oracle:
    image: duniter/duniter-v2s-g1-1000:1000-1.0.0
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

**Prérequis :** le fichier `node.key` (généré en A3) doit être présent à côté du `docker-compose.yml`.

```bash
# Injecter les clés de session (avant le premier démarrage)
# Le '--' bypasse l'entrypoint Docker
docker compose run --rm duniter-g1-smith -- key generate-session-keys \
  --chain g1 -d /var/lib/duniter --suri "<phrase secrète de l'étape A4>"

# Démarrer
docker compose up -d

# Vérifier que le Peer ID correspond à celui des specs (étape A7)
docker compose logs duniter-g1-smith | grep "Local node identity"
docker compose logs -f duniter-g1-smith | grep "Prepared block"
```

<details><summary>Variables d'environnement du docker-compose</summary>

- `DUNITER_VALIDATOR: "true"` : active le mode forgeron (production de blocs + GRANDPA).
- `DUNITER_PRUNING_PROFILE: light` : conserve uniquement les 256 derniers blocs d'état (suffisant pour un forgeron). Utiliser `archive` pour un miroir/indexeur.
- `DUNITER_PUBLIC_ADDR` : adresse P2P annoncée aux autres noeuds, doit correspondre au DNS public du serveur.
- `ORACLE_EXECUTION_INTERVAL: 1800` : l'oracle de distance s'exécute toutes les 30 min.
</details>

<details><summary>Alternative : build et lancement depuis les sources (sans Docker)</summary>

Si vous préférez ne pas utiliser Docker, vous pouvez compiler le binaire et lancer le nœud directement. Cette méthode ne nécessite ni les commandes xtask, ni de GitLab token.

**1. Récupérer la branche réseau et le rawspec :**

```bash
git fetch origin network/g1-1000:network/g1-1000
git checkout network/g1-1000

# Télécharger le rawspec depuis la page des releases GitLab
# URL disponible sur https://git.duniter.org/nodes/rust/duniter-v2s/-/releases
curl -o node/specs/g1-raw.json \
  "https://git.duniter.org/-/project/520/uploads/<hash>/g1-raw.json"
```

**2. Compiler le nœud et l'oracle de distance :**

```bash
cargo build --release --features g1,embed --no-default-features
cargo build --release -p distance-oracle --features g1,standalone,std --no-default-features
```

Le feature `embed` intègre le fichier `node/specs/g1-raw.json` dans le binaire à la compilation — il doit donc être présent **avant** le build. Cela permet d'utiliser `--chain g1` au lancement sans avoir le fichier JSON sur le serveur.

En CI, les builds utilisent désormais des dépendances vendored pour limiter l'usage réseau et disque sans dépendre de `-Zgit=shallow-deps`.

**3. Injecter les clés de session :**

```bash
./target/release/duniter key generate-session-keys \
  --chain g1 \
  -d /var/lib/duniter/g1 \
  --suri "<phrase secrète de l'étape A4>"
```

**4. Lancer en mode forgeron :**

```bash
./target/release/duniter \
  --chain g1 \
  -d /var/lib/duniter/g1 \
  --node-key-file /var/lib/duniter/g1/validator.key \
  --public-addr /dns/g1-boot1.duniter.org/tcp/30333/p2p/<PEER_ID> \
  --validator \
  --name g1-bootstrap \
  --state-pruning 1024 \
  --rpc-cors all \
  --rpc-methods unsafe \
  --no-telemetry \
  --no-prometheus
```

**5. Lancer l'oracle de distance** (dans un second terminal ou via systemd) :

```bash
while true; do
  ./target/release/distance-oracle \
    --evaluation-result-dir /var/lib/duniter/g1/chains/g1/distance/ \
    --rpc-url ws://127.0.0.1:9944
  echo "Attente 1800s avant prochaine exécution..."
  sleep 1800
done
```

Options notables :
- `--validator` : active le mode forgeron (équivalent de `DUNITER_VALIDATOR=true`).
- `--state-pruning 1024` : conserve les 1024 derniers blocs d'état. Utiliser `archive` pour un miroir/indexeur.
- `--rpc-methods unsafe` : nécessaire pour l'injection de clés et `author_rotateKeys`. Peut être retiré après la rotation.
- `--no-telemetry --no-prometheus` : optionnel, désactive la télémétrie et les métriques Prometheus.
- `-d /var/lib/duniter/g1` : répertoire de données du nœud. Le chemin de l'oracle (`--evaluation-result-dir`) doit pointer vers le sous-dossier `chains/g1/distance/` de ce répertoire.
</details>

### Étape 8 — Rotation des clés de session (optionnel)

Pour remplacer les clés du genesis par des clés générées sur le serveur :

```bash
curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"author_rotateKeys","params":[]}' \
  http://127.0.0.1:9944
```

Puis soumettre on-chain via `session.setKeys` ([Duniter Portal](https://duniter-portal.axiom-team.fr/) ou subxt). Prise d'effet après une epoch (4h).

<details><summary>Pourquoi faire la rotation</summary>

Les clés injectées à l'étape 7 proviennent de la machine de build. La rotation génère de nouvelles clés directement sur le serveur de production, ce qui évite que la phrase secrète de build ait un pouvoir de validation permanent.
</details>

### Étape 9 — Forgerons additionnels

Même docker-compose adapté (nom, adresse publique) + bootnode du bootstrap. Puis :

```bash
curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"author_rotateKeys","params":[]}' \
  http://127.0.0.1:9944
# Soumettre via session.setKeys
```

Alternative Debian : `dpkg -i duniter.deb` + configurer `/etc/duniter/env_file` + `systemctl start duniter-smith distance-oracle`.

<details><summary>Alternative : lancement depuis les sources</summary>

Même procédure de build que l'étape 7 (section « depuis les sources »), en adaptant `--name`, `--public-addr` et en ajoutant un bootnode :

```bash
./target/release/duniter \
  --chain g1 \
  -d /var/lib/duniter/g1 \
  --node-key-file /var/lib/duniter/g1/validator.key \
  --public-addr /dns/g1-smith2.duniter.org/tcp/30333/p2p/<PEER_ID> \
  --bootnodes /dns/g1-boot1.duniter.org/tcp/30333/p2p/<BOOT_PEER_ID> \
  --validator \
  --name g1-smith-2 \
  --state-pruning 1024 \
  --rpc-cors all \
  --rpc-methods unsafe
```

Ne pas oublier de lancer l'oracle de distance en parallèle (voir étape 7).
</details>

### Étape 10 — Noeuds miroirs

```yaml
services:
  duniter-g1-mirror:
    image: duniter/duniter-v2s-g1-1000:1000-1.0.0
    ports: ["9944:9944", "30333:30333"]
    environment:
      DUNITER_NODE_NAME: g1-mirror
      DUNITER_CHAIN_NAME: g1
      DUNITER_PRUNING_PROFILE: archive
      DUNITER_PUBLIC_RPC: wss://rpc.g1.duniter.org
    volumes: [g1-mirror:/var/lib/duniter]
```

<details><summary>Reverse proxy nginx pour WSS</summary>

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

Le `proxy_read_timeout 86400` (24h) est nécessaire pour les connexions WebSocket longue durée.
</details>

<details><summary>Alternative : lancement depuis les sources</summary>

Même procédure de build que l'étape 7 (section « depuis les sources »). Un nœud miroir ne nécessite ni `--validator`, ni oracle de distance, ni injection de clés de session.

```bash
./target/release/duniter \
  --chain g1 \
  -d /var/lib/duniter/g1 \
  --bootnodes /dns/g1-boot1.duniter.org/tcp/30333/p2p/<BOOT_PEER_ID> \
  --name g1-mirror \
  --state-pruning archive \
  --rpc-cors all \
  --rpc-external
```

- `--state-pruning archive` : conserve l'intégralité de l'état (nécessaire pour servir les requêtes RPC).
- `--rpc-external` : expose le RPC sur toutes les interfaces (nécessaire derrière un reverse proxy).
</details>

### Étape 11 — Build des images Docker squid (indexeur)

> **Prérequis :** Un noeud miroir avec RPC public (étape 10) doit être accessible, car le pipeline squid récupère les métadonnées substrate depuis le endpoint RPC configuré (ex : `wss://g1.p2p.legal/ws`).

```bash
cargo xtask release squid trigger-builds g1-1000

# Ou avec un noeud RPC spécifique (surcharge le défaut wss://g1.p2p.legal/ws) :
cargo xtask release squid trigger-builds g1-1000 --rpc-url wss://g1-rpc.mon-serveur.fr/ws
```

Cette commande déclenche le pipeline CI du projet [duniter-squid](https://git.duniter.org/nodes/duniter-squid) qui :
1. Télécharge les données genesis depuis la release `g1-1000`
2. Génère le fichier `genesis.json` et récupère les métadonnées substrate depuis le RPC (défaut par réseau, ou `--rpc-url`)
3. Build et push trois images Docker multi-arch (amd64+arm64) sur Docker Hub :
   - `duniter/squid-app-g1:<squid-version>` — processeur squid (indexeur blockchain)
   - `duniter/squid-graphile-g1:<squid-version>` — serveur GraphQL (PostGraphile)
   - `duniter/squid-postgres-g1:<squid-version>` — PostgreSQL avec wal2json

La version des images est celle du `package.json` du projet duniter-squid (ex : `0.5.8`), ou celle du tag git si le pipeline est déclenché par un push de tag (ex : `v0.5.8` → `0.5.8`).

<details><summary>Prérequis et variantes</summary>

- La variable `DOCKERHUB_TOKEN` doit être configurée dans les CI/CD variables du projet duniter-squid sur GitLab
- La variable `RELEASE_TAG` doit être configurée dans les CI/CD variables du projet duniter-squid (ex : `g1-1000`) pour le flow par push de tag. Le xtask la transmet automatiquement lors du déclenchement par API.
- Le `GITLAB_TOKEN` local doit avoir accès au projet `nodes/duniter-squid`

Pour builder depuis une branche spécifique du squid : `cargo xtask release squid trigger-builds g1-1000 --branch my-branch`

Pour utiliser un noeud RPC différent du défaut : `--rpc-url wss://mon-noeud.example.com/ws`. En déclenchement manuel depuis GitLab, ajouter la variable `RPC_URL` sur le job `prepare`.

Le pipeline peut aussi être déclenché par un push de tag (ex : `v0.5.8`) sur le repo squid. Dans ce cas, les jobs démarrent automatiquement et utilisent le tag comme version Docker.
</details>

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

- [ ] Blocs produits régulièrement
- [ ] Finalisation active
- [ ] Forgerons connectés et en ligne
- [ ] Oracle de distance opérationnel
- [ ] Identités et certifications migrées
- [ ] Masse monétaire correcte
- [ ] DU créé après 24h
- [ ] RPC public accessible
- [ ] Télémétrie visible
- [ ] Images Docker squid publiées sur Docker Hub

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
