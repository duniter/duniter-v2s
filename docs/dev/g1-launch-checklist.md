# Checklist de lancement Ğ1 v2 — Actions manuelles

Liste des actions à réaliser manuellement le jour J du lancement du réseau Ğ1 v2.

---

## 1. Fixer `first_ud` dans `resources/g1.yaml`

**Quand :** Juste avant le build final des specs réseau (étape 6 de la procédure g1-production-launch.md).

**Pourquoi :** Si laissé à `null`, le premier DU sera créé exactement 24h après le timestamp du premier bloc. Hors, le lancement peut avoir lieu à n'importe quelle heure. Pour conserver la cohérence avec la Ğ1 v1 (DU créé chaque jour à heure fixe), il faut fixer explicitement ce timestamp.

**Action :**

Mettre au 08/03/2026 à 11h00 UTC (12h00 heure de Paris, heure d'hiver) :
   ```yaml
   first_ud: 1772967600000
   ```

---

## 2. Définir `clique_smiths` dans `resources/g1.yaml`

**Quand :** En amont du jour J — nécessite coordination avec les forgerons volontaires.

**Pourquoi :** Ce champ liste les forgerons initiaux du réseau. Au genesis, chaque smith certifie automatiquement tous les autres (clique). Il faut minimum 3-5 forgerons pour assurer la finalisation GRANDPA (2/3 des validateurs).

**Action :**

1. Identifier les forgerons volontaires et leurs noms d'identité Ğ1
2. Désigner le forgeron bootstrap (celui qui produira les premiers blocs)
3. Générer ses clés de session et les renseigner dans le champ `session_keys`
4. Reporter la liste dans `resources/g1.yaml` :
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

## 3. Définir `technical_committee` dans `resources/g1.yaml`

**Quand :** En amont du jour J — nécessite décision communautaire / gouvernance.

**Pourquoi :** Le comité technique agit comme garde-fou via le sudo. Ses membres peuvent proposer et voter des extrinsics privilégiés (ex : upgrade runtime, correction d'urgence). C'est une décision de gouvernance, pas technique.

**Action :**

1. Décider de la composition du comité (recommandé : développeurs actifs + membres de confiance)
2. Reporter les noms dans `resources/g1.yaml` :
   ```yaml
   technical_committee: ["membre1", "membre2", "membre3", ...]
   ```

**Note :** Les membres du comité doivent être des identités présentes dans le genesis (soit migrées depuis la Ğ1 v1, soit dans `clique_smiths`).

---

*Ajouter les prochaines étapes au fur et à mesure de la préparation.*
