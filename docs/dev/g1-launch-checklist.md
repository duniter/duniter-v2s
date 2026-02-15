# Checklist de lancement Ğ1 v2 — Actions manuelles

Liste des actions à réaliser manuellement pour le lancement du réseau Ğ1 v2.
Organisée par étape de la procédure `g1-production-launch.md`.

---

## En amont du jour J

### A1. Définir `clique_smiths` dans `resources/g1.yaml`

**Pourquoi :** Ce champ liste les forgerons initiaux du réseau. Au genesis, chaque smith certifie automatiquement tous les autres (clique). Il faut minimum 4 forgerons pour satisfaire le seuil de 3 certifications smith.

**Action :**

1. Identifier les forgerons volontaires et leurs noms d'identité Ğ1
2. Vérifier que chaque forgeron est **membre actif** (adhésion non expirée dans le dump)
3. Désigner le forgeron bootstrap (celui qui produira les premiers blocs)
4. Générer ses clés de session (étape 4) et les renseigner dans le champ `session_keys`
5. Reporter la liste dans `resources/g1.yaml` :
   ```yaml
   clique_smiths:
     - name: "forgeron1"
     - name: "forgeron2"
     - name: "forgeron3"
     - name: "forgeron_bootstrap"
       session_keys: "0x..."
   ```

**Prérequis :** Chaque forgeron doit disposer d'un serveur prêt (4 Go RAM, 2 CPU, 100 Go SSD).

---

### A2. Définir `technical_committee` dans `resources/g1.yaml`

**Pourquoi :** Le comité technique agit comme garde-fou via le sudo. Ses membres peuvent proposer et voter des extrinsics privilégiés (ex : upgrade runtime, correction d'urgence). C'est une décision de gouvernance, pas technique.

**Action :**

1. Décider de la composition du comité (recommandé : développeurs actifs + membres de confiance)
2. Vérifier que chaque membre est **membre actif** dans le dump
3. Reporter les noms dans `resources/g1.yaml` :
   ```yaml
   technical_committee: ["membre1", "membre2", "membre3", ...]
   ```

---

### A3. Générer les clés réseau (node keys) des bootnodes

**Pourquoi :** Le Peer ID d'un nœud est dérivé de sa clé réseau (Ed25519). Il faut la générer en amont pour pouvoir renseigner les Peer ID dans les client specs **avant** le build-specs (étape 6), sans avoir à lancer de nœud.

**Action :**

Chaque opérateur de bootnode génère sa clé réseau :
```bash
# Génère une clé réseau + affiche le Peer ID
./target/release/duniter key generate-node-key --chain dev
# Ligne 1 = Peer ID (12D3KooW...)
# Ligne 2 = clé privée réseau (hex, 64 caractères)
```

**⚠️ Conserver précieusement la clé privée.** Elle sera injectée dans le nœud au démarrage via `--node-key <hex>` ou un fichier.

Pour retrouver le Peer ID à partir d'une clé existante :
```bash
echo "<clé privée hex>" > /tmp/node-key
./target/release/duniter key inspect-node-key --file /tmp/node-key
rm /tmp/node-key
```

---

### A4. Définir les `bootNodes` dans `node/specs/g1_client-specs.yaml`

**Pourquoi :** Ce sont les nœuds auxquels les clients se connecteront pour rejoindre le réseau. Les Peer ID proviennent des clés réseau générées en A3.

**Action :**

1. Collecter les Peer ID de chaque bootnode (générés en A3)
2. Reporter dans `node/specs/g1_client-specs.yaml` :
   ```yaml
   bootNodes:
     - "/dns/g1-boot1.duniter.org/tcp/30333/p2p/12D3KooW..."
     - "/dns/g1-boot2.duniter.org/tcp/30333/p2p/12D3KooW..."
   ```

---
