# Vérifier le code d'un runtime upgrade

Lors d'un vote au Technical Committee pour un runtime upgrade, chaque membre
doit vérifier que le hash blake2_256 de la preimage correspond bien au WASM
compilé depuis le code source publié. Sans cette vérification, un upgrade
malveillant pourrait se faire passer pour légitime.

## Etape 1 : Compiler le runtime avec srtool

Checkout la branche du runtime upgrade et compilez avec srtool :

```bash
git checkout runtime/<network>-<version>
cargo xtask release runtime build <network>
```

Exemple pour GTest 1100 :

```bash
git checkout runtime/gtest-1100
cargo xtask release runtime build gtest
```

Le WASM est généré dans `release/gtest_runtime.compact.compressed.wasm`.

La commande xtask utilise en interne le Docker `paritytech/srtool` dont la
version correspond au channel Rust dans `rust-toolchain.toml` (actuellement
`1.88.0`). Le build est reproductible : le même code source produit toujours
le même binaire WASM, quel que soit la machine.

<details><summary>Commande srtool manuelle (sans xtask)</summary>

```bash
docker run \
  -i \
  --rm \
  -e PACKAGE=<network>-runtime \
  -e RUNTIME_DIR=runtime/<network> \
  -v $PWD:/build \
  paritytech/srtool:1.88.0 build --app --json -cM
```

Le WASM est dans :
`runtime/<network>/target/srtool/release/wbuild/<network>-runtime/<network>_runtime.compact.compressed.wasm`

</details>

## Etape 2 : Récupérer le blake2_256 du build

```bash
cat release/srtool_output_<network>.json | jq -r '.runtimes.compressed.blake2_256'
```

Notez ce hash, c'est la référence de votre compilation locale.

## Etape 3 : Vérifier la preimage de la proposal

Sur Polkadot.js Apps connecté au réseau concerné :

1. Allez dans **Governance > Tech. committee > Proposals**
2. Identifiez la proposal de runtime upgrade
3. Vérifiez que la proposal référence bien une preimage contenant
   `upgradeOrigin.dispatchAsRootUncheckedWeight(system.setCode(...))`
4. Cliquez sur le lien de la preimage pour voir le `code: Bytes` du runtime
5. Vérifiez que le **hash de la preimage** correspond bien au call `system.setCode`
   référencé dans la proposal

### Calculer le blake2_256 depuis la preimage

Copiez le `code: Bytes` (la valeur hexadécimale complète du WASM, plusieurs Mo)
depuis la preimage et placez-le dans un script :

```bash
WASM_BYTES=0x42...

echo $WASM_BYTES | xxd -r -p | python3 -c \
  "import sys, hashlib; print('0x' + hashlib.blake2b(sys.stdin.buffer.read(), digest_size=32).hexdigest())"
```

Le hash blake2_256 affiché doit correspondre **exactement** à celui obtenu
à l'etape 2 via srtool.

## Resultat

Si les deux hash blake2_256 correspondent, vous avez la certitude que le code
source de la branche git produit bien le runtime proposé au vote. Vous pouvez
voter en confiance.

## Voir aussi

- [Procédure de release](./release.md) pour la création de la release GitLab
- [Post du forum](https://forum.duniter.org/t/technical-committee-check-runtime-upgrade-hash-compliance/13418) avec la procédure originale
