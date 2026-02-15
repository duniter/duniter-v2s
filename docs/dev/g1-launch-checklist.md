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

### A3. Définir les `bootNodes` dans `node/specs/g1_client-specs.yaml`

**Pourquoi :** Ce sont les nœuds auxquels les clients se connecteront pour rejoindre le réseau. Ils doivent être opérationnels dès le lancement du réseau, et leur Peer ID doit être renseigné dans les specs pour que les clients puissent s'y connecter.

**Action :**

1. Décider de la composition des bootnodes
2. Reporter les noms dans `node/specs/g1_client-specs.yaml` :
   ```yaml
   bootNodes:
     - "/dns/g1-boot1.duniter.org/tcp/30333/p2p/<PEER_ID>"
     - "/dns/g1-boot2.duniter.org/tcp/30333/p2p/<PEER_ID>"
     - "/dns/g1-boot3.duniter.org/tcp/30333/p2p/<PEER_ID>"
   ```

---

## Étape 3 — Fichiers de configuration réseau

### 3.1. Fixer `first_ud` dans `resources/g1.yaml`

**Pourquoi :** Si laissé à `null`, le premier DU sera créé exactement 24h après le timestamp du premier bloc. Hors, le lancement peut avoir lieu à n'importe quelle heure. Pour conserver la cohérence avec la Ğ1 v1 (DU créé chaque jour à heure fixe), il faut fixer explicitement ce timestamp.

**Action :**

Reporter dans `resources/g1.yaml` :
```yaml
first_ud: 1772967600000
```

---

## Étape 6 — Génération des specs réseau

### 6.1. Compléter `node/specs/g1_client-specs.yaml`

**Pourquoi :** Ce fichier contient le Peer ID du nœud bootstrap, qui n'est connu qu'après le déploiement du bootstrap (étape 9). Cependant, pour un premier build de test, un Peer ID temporaire peut être utilisé.

**Action (jour J, après étape 9) :**

1. Récupérer le Peer ID du nœud bootstrap :
   ```bash
   docker compose logs duniter-g1-smith | grep "Local node identity"
   ```
2. Mettre à jour `node/specs/g1_client-specs.yaml` :
   ```yaml
   bootNodes:
     - "/dns/g1-boot1.duniter.org/tcp/30333/p2p/12D3KooW..."
   ```
3. Relancer `cargo xtask release network build-specs g1` pour régénérer les specs avec le bon bootnode.

---

*Ajouter les prochaines étapes au fur et à mesure de la préparation.*
