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

Chaque opérateur de bootnode génère sa clé réseau (sur la machine de build) :
```bash
# Génère le fichier node.key + affiche le Peer ID
./target/release/duniter key generate-node-key --chain dev --file node.key
# → noter le Peer ID affiché (12D3KooW...) pour l'étape A6
# → transférer node.key sur le serveur bootstrap
```

**⚠️ Le fichier `node.key` doit être déployé sur le serveur** et monté dans le docker-compose (étape 7). Sans cela, l'entrypoint Docker génère une nouvelle clé et le Peer ID ne correspondra plus aux bootNodes des specs embarquées.

Pour retrouver le Peer ID à partir d'une clé existante :
```bash
./target/release/duniter key inspect-node-key --file node.key
```

---

### A4. Générer les clés de session du forgeron bootstrap

**Pourquoi :** Le forgeron bootstrap est le seul à produire des blocs au lancement. Ses clés de session doivent être générées et insérées dans `resources/g1.yaml` avant le build des specs.

**Action :**

```bash
# Générer une phrase secrète + clé publique SS58
./target/release/duniter key generate
# → noter la phrase secrète et la clé publique SS58

# Générer les clés de session à partir de la phrase secrète
# (--chain dev suffit : les clés dépendent de la phrase, pas du chain spec)
./target/release/duniter key generate-session-keys --chain dev --suri "<phrase secrète>"
# → coller le résultat hex (Session Keys: 0x...) dans resources/g1.yaml, champ session_keys
```

**⚠️ Conserver précieusement la phrase secrète.** Elle sera nécessaire pour la rotation des clés (étape 10).

---

### A5. Vérifier les paramètres économiques dans `resources/g1.yaml`

**Pourquoi :** Ces valeurs sont inscrites dans le bloc genesis et ne sont plus modifiables après lancement.

**Action :** Vérifier/ajuster dans `resources/g1.yaml` :

```yaml
ud: 1148                        # Montant du DU en centièmes (11,48 Ğ1)
first_ud: 1772967600000         # Timestamp ms du 1er DU (null = auto depuis migration)
first_ud_reeval: 1774112400000  # Timestamp ms de la 1ère réévaluation du DU
treasury_funder_pubkey: "2ny7..." # Clé publique v1 qui financera la trésorerie (1,00 Ğ1)
```

**Contrainte :** `first_ud` < `first_ud_reeval` et `ud` > 0 (vérifié au build).

---

### A6. Définir les `bootNodes` dans `node/specs/g1_client-specs.yaml`

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
