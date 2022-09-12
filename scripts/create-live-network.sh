#!/usr/bin/env bash
#
# USAGE
#
# 1. Generate genesis authorities session keys.
# 2. Create the json file that contains the genesis configuration and verify carefully that the
#    declared session keys correspond to the one you have generated in the first step.
# 3. Generate raw chain specs with script `gen-live-network-raw-spec.sh`.
# 4. Share the generated raw spec with other genesis authorities.
# 5. Each genesis authority should run this script with its session keys seed.
#

# This script is meant to be run on Unix/Linux based systems
set -e

# params
VALIDATOR_SESSION_KEYS_SURI=$1
CURRENCY="${2:-gdev}"
WORK_DIR="${3:-tmp/$CURRENCY}"
SPEC_DIR="${4:-resources}"

echo "CURRENCY=$CURRENCY"

# constants
DUNITER_IMAGE_TAG="sha-99fb985b"

# Clean and (re-)create working forders
rm -rf $WORK_DIR
mkdir -p $WORK_DIR/duniter-rpc
mkdir -p $WORK_DIR/duniter-validator/chains/$CURRENCY

# Helper to execute a duniter subcommand in docker
function duniter_tmp () {
  docker rm duniter-tmp > /dev/null
  OUTPUT=$(docker run --name duniter-tmp -it --entrypoint duniter duniter/duniter-v2s:$DUNITER_IMAGE_TAG "$@")
  echo "${OUTPUT::-1}"
}

if [ -e "$SPEC_DIR/$CURRENCY-raw.json" ]
then
  # copy raw chain spec
  cp $SPEC_DIR/$CURRENCY-raw.json $WORK_DIR/duniter-rpc/$CURRENCY-raw.json
  cp $SPEC_DIR/$CURRENCY-raw.json $WORK_DIR/duniter-validator/$CURRENCY-raw.json
else
  # generate raw chain spec
  echo "generate raw_chain spec…"
  export DUNITER_GENESIS_CONFIG="$SPEC_DIR/$CURRENCY.json"
  duniter_tmp build-spec --chain $CURRENCY-gl --raw > /var/lib/duniter/$CURRENCY-raw.json
  docker cp duniter-tmp:/var/lib/duniter/$CURRENCY-raw.json $WORK_DIR/duniter-rpc/$CURRENCY-raw.json
  cp $WORK_DIR/duniter-rpc/$CURRENCY-raw.json $WORK_DIR/duniter-validator/$CURRENCY-raw.json
fi

# generate rpc node key
echo "generate rpc node key…"
RPC_NODE_KEY=$(duniter_tmp key generate-node-key --file /var/lib/duniter/node-key.txt)
docker cp duniter-tmp:/var/lib/duniter/node-key.txt $WORK_DIR/duniter-rpc/node-key
echo "RPC_NODE_KEY=$RPC_NODE_KEY"

# generate validator node key
echo "generate validator node key…"
VALIDATOR_NODE_KEY=$(duniter_tmp key generate-node-key --file /var/lib/duniter/node-key.txt)
docker cp duniter-tmp:/var/lib/duniter/node-key.txt $WORK_DIR/duniter-validator/node-key
echo "VALIDATOR_NODE_KEY=$VALIDATOR_NODE_KEY"

# generate docker-compose file
echo "generate docker-compose file…"
cp docker/compose/live-template.docker-compose.yml $WORK_DIR/docker-compose.yml
sed -i -e "s/DUNITER_IMAGE_TAG/$DUNITER_IMAGE_TAG/g" $WORK_DIR/docker-compose.yml
sed -i -e "s/CURRENCY/$CURRENCY/g" $WORK_DIR/docker-compose.yml
sed -i -e "s/RPC_NODE_KEY/$RPC_NODE_KEY/g" $WORK_DIR/docker-compose.yml
sed -i -e "s/VALIDATOR_NODE_KEY/$VALIDATOR_NODE_KEY/g" $WORK_DIR/docker-compose.yml

# Inject validator session keys in validator node keystore
echo "Inject validator session keys in validator node keystore…"
duniter_tmp key generate-session-keys --chain "${CURRENCY}_local" --suri "$VALIDATOR_SESSION_KEYS_SURI" -d /var/lib/duniter
docker cp duniter-tmp:/var/lib/duniter/chains/${CURRENCY}_local/keystore $WORK_DIR/duniter-validator/chains/$CURRENCY

# Launch the network
echo "compose ready in '$WORK_DIR'"
cd $WORK_DIR
#docker compose up -d
