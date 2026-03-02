#!/usr/bin/env bash
#
# launch-network.sh — Automated launch of a Duniter v2s network (g1, gtest, gdev)
#
# Usage: ./scripts/launch-network.sh [OPTIONS]
#
# Options:
#   --runtime NAME     Runtime: g1|gtest|gdev (default: g1)
#   --resume-from N    Resume from step N (1-11)
#   --skip-build       Skip long builds (assumes release/ is populated)
#   --dump-url URL     Custom G1 v1 dump URL
#   -h, --help         Show help
#
# Required environment variables:
#   GITLAB_TOKEN         GitLab Access Token (scope: api)
#   DUNITERTEAM_PASSWD   Docker Hub password for org duniter

set -euo pipefail

# ============================================================================
# Constants
# ============================================================================

GITLAB_PROJECT_URL="https://git.duniter.org/api/v4/projects/nodes%2Frust%2Fduniter-v2s"
GITLAB_PROJECT_ID="520"
TOTAL_STEPS=11
DEPLOY_DIR="deploy-bootstrap"
DUNITER_BIN="./target/release/duniter"
SECRETS_FILE=""

# yq compatibility: Python yq (v3) needs -r for raw output, mikefarah/yq (v4) does not
YQ_RAW=""
if yq --version 2>&1 | grep -qE "^yq [0-3]\.|jq"; then
  YQ_RAW="-r"  # Python yq (jq wrapper)
fi

# Wrapper: yq_raw <expression> <file> — outputs raw strings (no JSON quotes)
yq_raw() { yq $YQ_RAW "$1" "$2"; }

# ============================================================================
# 1. Utilities
# ============================================================================

# Colors (disabled if not a terminal)
if [ -t 1 ]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  BLUE='\033[0;34m'
  CYAN='\033[0;36m'
  BOLD='\033[1m'
  NC='\033[0m'
else
  RED='' GREEN='' YELLOW='' BLUE='' CYAN='' BOLD='' NC=''
fi

die()   { echo -e "${RED}ERROR: $*${NC}" >&2; exit 1; }
info()  { echo -e "${BLUE}INFO:${NC} $*"; }
warn()  { echo -e "${YELLOW}WARN:${NC} $*"; }
ok()    { echo -e "${GREEN}OK:${NC} $*"; }

STEP_START_TIME=""
CURRENT_STEP=0

step_header() {
  local n="$1"; shift
  CURRENT_STEP="$n"

  # Show elapsed time for previous step
  if [ -n "$STEP_START_TIME" ]; then
    local elapsed=$(( $($DATE_CMD +%s) - STEP_START_TIME ))
    local mins=$((elapsed / 60))
    local secs=$((elapsed % 60))
    ok "Previous step completed in ${mins}m${secs}s"
    echo ""
  fi

  STEP_START_TIME=$($DATE_CMD +%s)
  echo -e "${BOLD}${CYAN}======================================${NC}"
  echo -e "${BOLD}${CYAN} Step ${n}/${TOTAL_STEPS}: $*${NC}"
  echo -e "${BOLD}${CYAN}======================================${NC}"
  echo ""
}

confirm_or_die() {
  local msg="${1:-Continue?}"
  echo -en "${YELLOW}${msg} [Y/n] ${NC}"
  read -r answer
  case "${answer:-Y}" in
    [Yy]*) return 0 ;;
    *) die "Aborted by user." ;;
  esac
}

prompt_with_default() {
  local prompt_msg="$1"
  local default_val="$2"
  echo -en "${CYAN}${prompt_msg} [${default_val}]: ${NC}" >&2
  read -r answer
  echo "${answer:-$default_val}"
}

prompt_secret() {
  local prompt_msg="$1"
  echo -en "${CYAN}${prompt_msg}: ${NC}" >&2
  read -rs answer
  echo "" >&2
  echo "$answer"
}

require_cmd() {
  local cmd="$1"
  local hint="${2:-}"
  if ! command -v "$cmd" &>/dev/null; then
    if [ -n "$hint" ]; then
      die "Required command '$cmd' not found. $hint"
    else
      die "Required command '$cmd' not found."
    fi
  fi
}

require_env() {
  local var_name="$1"
  local hint="${2:-}"
  if [ -z "${!var_name:-}" ]; then
    if [ -n "$hint" ]; then
      die "Required environment variable '$var_name' is not set. $hint"
    else
      die "Required environment variable '$var_name' is not set."
    fi
  fi
}

# ============================================================================
# 2. Version detection
# ============================================================================

detect_spec_version() {
  local runtime_lib="runtime/${RUNTIME}/src/lib.rs"
  [ -f "$runtime_lib" ] || die "Runtime lib not found: $runtime_lib"

  SPEC_VERSION=$(grep 'spec_version:' "$runtime_lib" | head -1 | grep -o '[0-9]\+')
  [ -n "$SPEC_VERSION" ] || die "Could not detect spec_version from $runtime_lib"
  ok "spec_version: $SPEC_VERSION"
}

detect_client_version() {
  local cargo_toml="node/Cargo.toml"
  [ -f "$cargo_toml" ] || die "Node Cargo.toml not found: $cargo_toml"

  CLIENT_VERSION=$(grep '^version' "$cargo_toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
  [ -n "$CLIENT_VERSION" ] || die "Could not detect client version from $cargo_toml"
  ok "client_version: $CLIENT_VERSION"
}

derive_names() {
  NETWORK_TAG="${RUNTIME}-${SPEC_VERSION}"
  NETWORK_BRANCH="network/${NETWORK_TAG}"
  DOCKER_IMAGE="duniter/duniter-v2s-${NETWORK_TAG}"
  DOCKER_TAG="${SPEC_VERSION}-${CLIENT_VERSION}"
  CLIENT_MILESTONE="client-${CLIENT_VERSION}"

  echo ""
  info "Derived configuration:"
  echo "  NETWORK_TAG      = $NETWORK_TAG"
  echo "  NETWORK_BRANCH   = $NETWORK_BRANCH"
  echo "  DOCKER_IMAGE     = $DOCKER_IMAGE"
  echo "  DOCKER_TAG       = $DOCKER_TAG"
  echo "  CLIENT_MILESTONE = $CLIENT_MILESTONE"
  echo ""
}

resolve_dump_url() {
  if [ -n "${DUMP_URL:-}" ]; then
    RESOLVED_DUMP_URL="$DUMP_URL"
    return 0
  fi

  # Same logic as xtask: try today, then yesterday (backup generated at 23:00 UTC)
  local base="https://dl.cgeek.fr/public/auto-backup-g1-duniter-1.8.7"
  local today yesterday today_url yesterday_url

  today=$($DATE_CMD -u +%Y-%m-%d)
  today_url="${base}_${today}_23-00.tgz"

  if curl -s --head --fail --location "$today_url" -o /dev/null 2>/dev/null; then
    RESOLVED_DUMP_URL="$today_url"
  else
    yesterday=$($DATE_CMD -u -d "yesterday" +%Y-%m-%d 2>/dev/null \
      || $DATE_CMD -u -v-1d +%Y-%m-%d 2>/dev/null)
    yesterday_url="${base}_${yesterday}_23-00.tgz"
    if curl -s --head --fail --location "$yesterday_url" -o /dev/null 2>/dev/null; then
      RESOLVED_DUMP_URL="$yesterday_url"
      warn "Today's dump not available, using yesterday's ($yesterday)"
    else
      warn "No G1v1 dump found for today ($today) or yesterday ($yesterday)."
      warn "Provide one manually with --dump-url or check https://dl.cgeek.fr/public/"
      RESOLVED_DUMP_URL="${yesterday_url} (NOT VERIFIED)"
    fi
  fi
}

# ============================================================================
# 3. Validations
# ============================================================================

validate_prerequisites() {
  info "Checking required commands..."

  require_cmd git
  require_cmd cargo
  require_cmd docker "Install Docker: https://docs.docker.com/get-docker/"
  require_cmd curl
  require_cmd jq "Install jq: https://stedolan.github.io/jq/download/"
  require_cmd yq "Install yq: https://github.com/mikefarah/yq"

  ok "All required commands found."

  info "Checking environment variables..."
  require_env GITLAB_TOKEN "Export GITLAB_TOKEN with scope 'api'."
  require_env DUNITERTEAM_PASSWD "Export DUNITERTEAM_PASSWD (Docker Hub org duniter)."
  ok "Environment variables set."

  info "Validating GitLab token..."
  local http_code
  http_code=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
    "${GITLAB_PROJECT_URL}")
  [ "$http_code" = "200" ] || die "GitLab token validation failed (HTTP $http_code). Check GITLAB_TOKEN scope."
  ok "GitLab token valid."

  info "Checking Docker daemon..."
  docker info &>/dev/null || die "Docker daemon is not running. Start Docker and retry."
  ok "Docker daemon running."

  info "Checking Docker Hub login (org duniter)..."
  echo "${DUNITERTEAM_PASSWD}" | docker login -u duniterteam --password-stdin &>/dev/null \
    || die "Docker Hub login failed. Check DUNITERTEAM_PASSWD."

  # Verify the account can actually access the duniter org by querying an existing repo
  local hub_token
  hub_token=$(curl -s "https://hub.docker.com/v2/users/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"duniterteam\",\"password\":\"${DUNITERTEAM_PASSWD}\"}" \
    | jq -r '.token // empty')
  if [ -z "$hub_token" ]; then
    die "Docker Hub API login failed. Check DUNITERTEAM_PASSWD."
  fi

  local org_check
  org_check=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "Authorization: Bearer ${hub_token}" \
    "https://hub.docker.com/v2/orgs/duniter/")
  if [ "$org_check" != "200" ]; then
    die "Account 'duniterteam' cannot access Docker Hub org 'duniter' (HTTP $org_check). Check permissions."
  fi
  ok "Docker Hub login successful (org duniter accessible)."

  # Check that no release with this spec_version already exists on GitLab
  # Must check all tag formats: network release + runtime release (old and new format)
  info "Checking that spec_version ${SPEC_VERSION} is not already released..."
  local release_tags_to_check=(
    "${NETWORK_TAG}"                        # network release: g1-1000
    "runtime-${SPEC_VERSION}"               # runtime release old format: runtime-1000
    "runtime-${RUNTIME}-${SPEC_VERSION}"    # runtime release new format: runtime-g1-1000
  )
  for tag in "${release_tags_to_check[@]}"; do
    local release_code
    release_code=$(curl -s -o /dev/null -w "%{http_code}" \
      -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
      "https://git.duniter.org/api/v4/projects/${GITLAB_PROJECT_ID}/releases/${tag}")
    if [ "$release_code" = "200" ]; then
      die "GitLab release '${tag}' already exists. Bump spec_version in runtime/${RUNTIME}/src/lib.rs before launching a new network."
    fi
  done
  ok "No existing release found for spec_version ${SPEC_VERSION}."

  # Check that no client release with this client_version already exists (any network)
  info "Checking that client_version ${CLIENT_VERSION} is not already released..."
  local existing_client_tag
  existing_client_tag=$(curl -s -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
    "https://git.duniter.org/api/v4/projects/${GITLAB_PROJECT_ID}/releases?per_page=100" \
    | jq -r --arg cv "-${CLIENT_VERSION}" '[.[].tag_name | select(endswith($cv))] | first // empty')
  if [ -n "$existing_client_tag" ]; then
    die "Client version ${CLIENT_VERSION} already released as '${existing_client_tag}'. Bump version in node/Cargo.toml."
  fi
  ok "No existing release found for client_version ${CLIENT_VERSION}."
}

validate_g1_yaml() {
  local yaml_file="resources/${RUNTIME}.yaml"
  [ -f "$yaml_file" ] || die "Genesis config not found: $yaml_file"

  info "Validating $yaml_file..."

  # Count smiths
  local smith_count
  smith_count=$(yq '.smiths | length' "$yaml_file")
  [ "$smith_count" -ge 4 ] || die "Need at least 4 smiths in $yaml_file (found $smith_count). Required for 3-cert smith threshold."
  ok "Smith count: $smith_count (>= 4)"

  # Check that at least one smith has session_keys
  local has_session_keys
  has_session_keys=$(yq '[.smiths[] | select(.session_keys)] | length' "$yaml_file")
  [ "$has_session_keys" -ge 1 ] || die "At least one smith must have session_keys in $yaml_file (the bootstrap smith)."
  ok "Bootstrap smith with session_keys found."

  # Check technical_committee
  local tc_count
  tc_count=$(yq '.technical_committee | length' "$yaml_file")
  [ "$tc_count" -ge 1 ] || die "technical_committee is empty in $yaml_file."
  ok "Technical committee: $tc_count members."

  # Check economic parameters
  local ud first_ud first_ud_reeval treasury_funder
  ud=$(yq '.ud' "$yaml_file")
  first_ud=$(yq '.first_ud' "$yaml_file")
  first_ud_reeval=$(yq '.first_ud_reeval' "$yaml_file")
  treasury_funder=$(yq '.treasury_funder_pubkey' "$yaml_file")

  [ "$ud" != "null" ] && [ "$ud" -gt 0 ] 2>/dev/null || die "ud must be > 0 in $yaml_file (got: $ud)"
  [ "$first_ud" != "null" ] || warn "first_ud is null (will be auto-computed from migration data)"
  [ "$first_ud_reeval" != "null" ] || warn "first_ud_reeval is null in $yaml_file"
  [ "$treasury_funder" != "null" ] && [ -n "$treasury_funder" ] || warn "treasury_funder_pubkey not set in $yaml_file"
  ok "Economic parameters validated (ud=$ud)."
}

validate_client_specs() {
  local specs_file="node/specs/${RUNTIME}_client-specs.yaml"
  [ -f "$specs_file" ] || die "Client specs not found: $specs_file"

  info "Validating $specs_file..."

  # Check bootNodes with valid Peer ID
  local boot_count
  boot_count=$(yq '.bootNodes | length' "$specs_file")
  [ "$boot_count" -ge 1 ] || die "No bootNodes defined in $specs_file."

  local has_peer_id
  has_peer_id=$(yq '.bootNodes[]' "$specs_file" | grep -c '/p2p/12D3KooW' || true)
  [ "$has_peer_id" -ge 1 ] || warn "No bootNodes with valid Peer ID (/p2p/12D3KooW...) in $specs_file."
  ok "bootNodes: $boot_count entries."

  # Warn if TODO present
  if grep -qi 'TODO' "$specs_file"; then
    warn "TODO found in $specs_file — review before launch!"
  fi
}

# ============================================================================
# 3b. Bootstrap key setup (checklist A3, A4, A7)
# ============================================================================

setup_bootstrap_keys() {
  local yaml_file="resources/${RUNTIME}.yaml"
  local specs_file="node/specs/${RUNTIME}_client-specs.yaml"

  echo ""
  echo -e "${BOLD}${CYAN}======================================${NC}"
  echo -e "${BOLD}${CYAN} Bootstrap key setup${NC}"
  echo -e "${BOLD}${CYAN}======================================${NC}"
  echo ""

  # Check for duniter binary
  if [ ! -x "$DUNITER_BIN" ]; then
    warn "Duniter binary not found at $DUNITER_BIN."
    warn "Cannot auto-generate keys. Build first: cargo build --release"
    warn "Ensure session_keys and bootNodes are set manually (checklist A3, A4, A7)."
    confirm_or_die "Continue without auto-generating keys?"
    return 0
  fi

  # Idempotency: if chain spec is already built and a matching secrets file exists, skip
  local chain_spec="release/network/${RUNTIME}.json"
  if [ -f "$chain_spec" ]; then
    # Find a secrets file that matches the existing chain spec
    local genesis_babe
    genesis_babe=$(python3 -c "
import json
spec = json.load(open('$chain_spec'))
keys_list = spec['genesis']['runtimeGenesis']['patch']['session']['keys']
for entry in keys_list:
    keys = entry[2] if len(entry) > 2 else entry[1]
    vals = [keys.get(k,'') for k in ['grandpa','babe','im_online','authority_discovery']]
    if len(set(vals)) > 1:
        print(keys['babe'])
        break
" 2>/dev/null)
    if [ -n "$genesis_babe" ]; then
      for sf in $(ls -t launch-secrets-${NETWORK_TAG}-*.txt 2>/dev/null); do
        local sf_ss58
        sf_ss58=$(grep 'SS58 Address:' "$sf" 2>/dev/null | head -1 | sed 's/.*SS58 Address: *//' || true)
        if [ -n "$sf_ss58" ] && [ "$sf_ss58" = "$genesis_babe" ]; then
          SECRETS_FILE="$sf"
          ok "Chain spec already built. Keys match secrets file: $SECRETS_FILE"
          warn "Skipping key regeneration (would invalidate the existing chain spec/Docker image)."
          warn "To force regeneration, delete $chain_spec first."
          return 0
        fi
      done
      warn "Chain spec exists but no matching secrets file found. Regenerating keys..."
      warn "You will need to rebuild the chain spec (steps 4-6) after this."
      confirm_or_die "Continue with key regeneration?"
    fi
  fi

  # Init secrets file
  SECRETS_FILE="launch-secrets-${NETWORK_TAG}-$($DATE_CMD +%Y%m%d-%H%M%S).txt"
  {
    echo "============================================"
    echo " Duniter v2s Launch Secrets"
    echo " Network: ${NETWORK_TAG}"
    echo " Generated: $($DATE_CMD -Iseconds 2>/dev/null || $DATE_CMD +%Y-%m-%dT%H:%M:%S)"
    echo "============================================"
    echo ""
    echo "KEEP THIS FILE SECURE — contains secret keys!"
    echo ""
  } > "$SECRETS_FILE"

  # --- Node key (checklist A3) — always regenerate ---
  local node_key_file="${DEPLOY_DIR}/node.key"
  local peer_id=""

  mkdir -p "$DEPLOY_DIR"

  info "Generating node key (checklist A3)..."
  peer_id=$("$DUNITER_BIN" key generate-node-key --chain dev --file "$node_key_file" 2>&1)
  ok "Node key generated: $node_key_file"
  ok "Peer ID: $peer_id"

  {
    echo "--- Node Key (checklist A3) ---"
    echo "File:    $node_key_file"
    echo "Peer ID: $peer_id"
    echo "Key:     $(cat "$node_key_file")"
    echo ""
  } >> "$SECRETS_FILE"

  # --- Update bootNodes in client-specs (checklist A7) ---
  local dns_host
  dns_host=$(prompt_with_default "DNS hostname for bootstrap bootNode" "g1-bootstrap.p2p.legal")
  local new_bootnode="/dns/${dns_host}/tcp/30333/p2p/${peer_id}"

  info "Updating bootNodes in $specs_file..."
  info "  New: $new_bootnode"
  local new_bootnode_line="  - \"${new_bootnode}\""
  awk -v new="$new_bootnode_line" '
    /^  # TODO.*/ && prev == "bootNodes:" { next }
    /^  - "\/dns\// { print new; next }
    { prev=$0; print }
  ' "$specs_file" > "${specs_file}.tmp" && mv "${specs_file}.tmp" "$specs_file"
  ok "bootNodes updated in $specs_file"

  {
    echo "--- BootNode (checklist A7) ---"
    echo "File:     $specs_file"
    echo "DNS:      $dns_host"
    echo "BootNode: $new_bootnode"
    echo ""
  } >> "$SECRETS_FILE"

  # --- Session keys (checklist A4) — always regenerate ---
  # Find which smith currently has session_keys (the bootstrap smith)
  local bootstrap_smith
  bootstrap_smith=$(yq_raw '.smiths | to_entries | map(select(.value.session_keys)) | .[0].key // ""' "$yaml_file")

  if [ -z "$bootstrap_smith" ]; then
    # No smith has session_keys yet — ask which one is the bootstrap
    local smith_list
    smith_list=$(yq_raw '.smiths | keys | .[]' "$yaml_file")
    echo ""
    info "Which smith is the bootstrap (first block producer)?"
    echo "$smith_list" | nl -ba
    echo ""
    local smith_num
    smith_num=$(prompt_with_default "Smith number" "1")
    bootstrap_smith=$(echo "$smith_list" | sed -n "${smith_num}p")
    [ -n "$bootstrap_smith" ] || die "Invalid smith number."
  else
    info "Current bootstrap smith: $bootstrap_smith"
  fi

  info "Generating session keys for '$bootstrap_smith' (checklist A4)..."

  # Generate a new account (secret phrase)
  local keygen_output
  keygen_output=$("$DUNITER_BIN" key generate -w 12 2>&1)

  local secret_phrase
  secret_phrase=$(echo "$keygen_output" | grep 'Secret phrase:' | sed 's/.*Secret phrase: *//')
  local ss58_address
  ss58_address=$(echo "$keygen_output" | grep 'SS58 Address:' | sed 's/.*SS58 Address: *//')

  [ -n "$secret_phrase" ] || die "Failed to generate secret phrase."

  # Generate session keys from the secret phrase
  local tmpdir
  tmpdir=$(mktemp -d)
  local session_output
  session_output=$("$DUNITER_BIN" key generate-session-keys \
    --chain dev --suri "$secret_phrase" -d "$tmpdir" 2>&1)
  rm -rf "$tmpdir"

  local session_keys_hex
  session_keys_hex=$(echo "$session_output" | grep 'Session Keys:' | sed 's/.*Session Keys: *//')
  [ -n "$session_keys_hex" ] || die "Failed to generate session keys."

  ok "Session keys generated: ${session_keys_hex:0:20}..."

  # Replace session_keys in-place using sed (preserves comments and formatting)
  if grep -q '^ *session_keys:' "$yaml_file"; then
    # Replace the existing session_keys line value
    sed "s|^ *session_keys: \"0x[a-f0-9]*\"|    session_keys: \"${session_keys_hex}\"|" \
      "$yaml_file" > "${yaml_file}.tmp" && mv "${yaml_file}.tmp" "$yaml_file"
  else
    # No session_keys line exists — insert after bootstrap smith's certs_received
    awk -v keys="$session_keys_hex" '
      { print }
      /^  "'"$bootstrap_smith"'":/ { in_target = 1 }
      in_target == 1 && /^    certs_received:/ {
        printf "    session_keys: \"%s\"\n", keys
        in_target = 0
      }
    ' "$yaml_file" > "${yaml_file}.tmp" && mv "${yaml_file}.tmp" "$yaml_file"
  fi
  ok "session_keys replaced for '$bootstrap_smith' in $yaml_file"

  {
    echo "--- Session Keys (checklist A4) ---"
    echo "File:            $yaml_file"
    echo "Bootstrap smith: $bootstrap_smith"
    echo "Secret phrase:   $secret_phrase"
    echo "SS58 Address:    $ss58_address"
    echo "Session Keys:    $session_keys_hex"
    echo ""
    echo "Key details:"
    echo "$session_output"
    echo ""
    echo "IMPORTANT: Keep the secret phrase safe!"
    echo "  It is needed for session key rotation (step 8)"
    echo "  and for 'docker compose run ... key generate-session-keys --suri' (step 7)"
    echo ""
  } >> "$SECRETS_FILE"

  _finalize_secrets_file
}

_finalize_secrets_file() {
  if [ -n "$SECRETS_FILE" ] && [ -f "$SECRETS_FILE" ]; then
    chmod 600 "$SECRETS_FILE"
    echo ""
    ok "Secrets saved to: $SECRETS_FILE"
    warn "This file contains sensitive keys. Store it securely and delete after launch."
    echo ""
  fi
}

# ============================================================================
# 4. Step functions
# ============================================================================

step1_create_network_branch() {
  step_header 1 "Create network branch"

  # Check if branch exists locally
  if git rev-parse --verify "$NETWORK_BRANCH" &>/dev/null; then
    info "Branch '$NETWORK_BRANCH' already exists locally."
    confirm_or_die "Switch to existing branch?"
    git checkout "$NETWORK_BRANCH"
  else
    # Check if branch exists on remote
    if git ls-remote --heads origin "$NETWORK_BRANCH" | grep -q "$NETWORK_BRANCH"; then
      info "Branch '$NETWORK_BRANCH' exists on remote. Checking out..."
      git checkout -b "$NETWORK_BRANCH" "origin/$NETWORK_BRANCH"
    else
      info "Creating branch '$NETWORK_BRANCH' from master..."
      git checkout master
      git pull origin master
      git checkout -b "$NETWORK_BRANCH"
    fi
  fi

  # Ensure release directories exist
  mkdir -p release/network release/client

  # Verify spec_version in source
  local current_sv
  current_sv=$(grep 'spec_version:' "runtime/${RUNTIME}/src/lib.rs" | head -1 | grep -o '[0-9]\+')
  [ "$current_sv" = "$SPEC_VERSION" ] || die "spec_version mismatch: expected $SPEC_VERSION, found $current_sv"
  ok "spec_version $SPEC_VERSION confirmed in source."

  # Verify client version
  local current_cv
  current_cv=$(grep '^version' "node/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
  [ "$current_cv" = "$CLIENT_VERSION" ] || die "Client version mismatch: expected $CLIENT_VERSION, found $current_cv"
  ok "client_version $CLIENT_VERSION confirmed in node/Cargo.toml."

  # Check required files from checklist
  [ -f "resources/${RUNTIME}.yaml" ] || die "Missing resources/${RUNTIME}.yaml"
  [ -f "node/specs/${RUNTIME}_client-specs.yaml" ] || die "Missing node/specs/${RUNTIME}_client-specs.yaml"
  ok "Required config files present."

  # Commit all modified config files (checklist + bootstrap key setup)
  local files_to_commit=(
    "resources/${RUNTIME}.yaml"
    "node/specs/${RUNTIME}_client-specs.yaml"
    "runtime/${RUNTIME}/src/lib.rs"
    "node/Cargo.toml"
    "Cargo.lock"
    ".gitignore"
    ".gitlab-ci.yml"
  )
  local has_changes=false
  for f in "${files_to_commit[@]}"; do
    if [ -f "$f" ] && ! git diff --quiet -- "$f" 2>/dev/null; then
      has_changes=true
      break
    fi
  done

  if [ "$has_changes" = true ]; then
    info "Staged files for commit:"
    for f in "${files_to_commit[@]}"; do
      if [ -f "$f" ] && ! git diff --quiet -- "$f" 2>/dev/null; then
        echo "  $f"
        git add "$f"
      fi
    done
    confirm_or_die "Commit these files to branch '$NETWORK_BRANCH'?"
    git commit -m "chore(${RUNTIME}): configure ${NETWORK_TAG}"
    ok "Changes committed."
  else
    info "No pending changes to commit."
  fi

  # Push branch to remote
  if ! git ls-remote --heads origin "$NETWORK_BRANCH" | grep -q "$NETWORK_BRANCH"; then
    info "Pushing new branch to remote..."
    confirm_or_die "Push '$NETWORK_BRANCH' to origin?"
    git push -u origin "$NETWORK_BRANCH"
    ok "Branch pushed."
  else
    # Branch exists but we may have new commits to push
    local local_rev remote_rev
    local_rev=$(git rev-parse HEAD)
    remote_rev=$(git rev-parse "origin/$NETWORK_BRANCH" 2>/dev/null || echo "")
    if [ "$local_rev" != "$remote_rev" ]; then
      info "Local branch is ahead of remote."
      confirm_or_die "Push changes to origin/$NETWORK_BRANCH?"
      git push origin "$NETWORK_BRANCH"
      ok "Changes pushed."
    else
      ok "Branch up to date with remote."
    fi
  fi
}

step2_g1_migration_data() {
  step_header 2 "G1 migration data"

  if [ -f "release/network/genesis.json" ]; then
    info "release/network/genesis.json already exists."
    confirm_or_die "Skip migration data generation?"
    return 0
  fi

  local dump_args=()
  if [ -n "${DUMP_URL:-}" ]; then
    dump_args=(--dump-url "$DUMP_URL")
  fi

  info "Running: cargo xtask release network g1-data ${dump_args[*]:-}"
  cargo xtask release network g1-data "${dump_args[@]+"${dump_args[@]}"}"

  # Verify output
  [ -f "release/network/genesis.json" ] || die "genesis.json was not generated."

  local identity_count monetary_mass
  identity_count=$(jq '.identities | length' release/network/genesis.json)
  monetary_mass=$(jq '.initial_monetary_mass' release/network/genesis.json)

  ok "Migration data generated:"
  echo "  Identities:    $identity_count"
  echo "  Monetary mass: $monetary_mass"
}

step3_build_wasm_runtime() {
  step_header 3 "Build WASM runtime"

  local wasm_file="release/network/${RUNTIME}_runtime.compact.compressed.wasm"

  if [ -f "$wasm_file" ]; then
    info "$wasm_file already exists."
    confirm_or_die "Skip WASM build?"
    return 0
  fi

  if [ "${SKIP_BUILD:-false}" = "true" ]; then
    die "WASM runtime not found and --skip-build is set. Build manually first."
  fi

  warn "This step uses srtool (Docker) and may take a long time."
  warn "On ARM Macs, Docker Desktop needs 16 GB+ RAM allocated."
  confirm_or_die "Start WASM build?"

  info "Running: cargo xtask release network build-runtime $RUNTIME"
  cargo xtask release network build-runtime "$RUNTIME"

  [ -f "$wasm_file" ] || die "WASM file not generated: $wasm_file"

  local wasm_size
  wasm_size=$(wc -c < "$wasm_file" | tr -d ' ')
  ok "WASM runtime built: ${wasm_size} bytes"
}

step4_build_network_specs() {
  step_header 4 "Build network specs"

  local specs_file="release/network/${RUNTIME}.json"

  if [ -f "$specs_file" ]; then
    info "$specs_file already exists."
    confirm_or_die "Skip specs build?"
    return 0
  fi

  if [ "${SKIP_BUILD:-false}" = "true" ]; then
    die "Network specs not found and --skip-build is set. Build manually first."
  fi

  info "Running: cargo xtask release network build-specs $RUNTIME"
  cargo xtask release network build-specs "$RUNTIME"

  [ -f "$specs_file" ] || die "Specs file not generated: $specs_file"
  ok "Network specs built: $specs_file"
}

step5_network_release() {
  step_header 5 "Network release (GitLab)"

  # Verify required files
  local required_files=(
    "release/network/genesis.json"
    "release/network/${RUNTIME}.json"
    "release/network/${RUNTIME}_runtime.compact.compressed.wasm"
  )

  for f in "${required_files[@]}"; do
    [ -f "$f" ] || die "Missing required file: $f"
  done

  # Check if release already exists
  local http_code
  http_code=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
    "https://git.duniter.org/api/v4/projects/${GITLAB_PROJECT_ID}/releases/${NETWORK_TAG}")

  if [ "$http_code" = "200" ]; then
    info "GitLab release '${NETWORK_TAG}' already exists."
    confirm_or_die "Skip network release creation?"
    return 0
  fi

  info "Running: cargo xtask release network create $NETWORK_TAG $NETWORK_BRANCH"
  confirm_or_die "Create GitLab network release '${NETWORK_TAG}'?"
  cargo xtask release network create "$NETWORK_TAG" "$NETWORK_BRANCH"

  ok "Network release '${NETWORK_TAG}' created on GitLab."
}

step6_client_release() {
  step_header 6 "Client release"

  local client_release_tag="${NETWORK_TAG}-${CLIENT_VERSION}"

  # Check if client release already exists
  local http_code
  http_code=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
    "https://git.duniter.org/api/v4/projects/${GITLAB_PROJECT_ID}/releases/${client_release_tag}")

  if [ "$http_code" = "200" ]; then
    info "Client release '${client_release_tag}' already exists on GitLab."
    confirm_or_die "Skip to CI trigger?"

    # Still offer to trigger builds (they may have failed previously)
    echo ""
    warn "The next command triggers CI builds which take ~30-90 minutes."
    echo "  Docker image: ${DOCKER_IMAGE}:${DOCKER_TAG}"
    echo ""
    confirm_or_die "Trigger CI builds?"

    info "Running: cargo xtask release client trigger-builds $NETWORK_TAG $NETWORK_BRANCH"
    cargo xtask release client trigger-builds "$NETWORK_TAG" "$NETWORK_BRANCH"
    ok "CI builds triggered. Docker image will be: ${DOCKER_IMAGE}:${DOCKER_TAG}"
  else
    echo ""
    warn "Before continuing, ensure the GitLab milestone '${CLIENT_MILESTONE}' exists."
    echo "  Create it at: https://git.duniter.org/nodes/rust/duniter-v2s/-/milestones/new"
    echo "  Title: ${CLIENT_MILESTONE}"
    echo ""
    confirm_or_die "Is the milestone '${CLIENT_MILESTONE}' created on GitLab?"

    # Sub-step: build raw specs
    info "Running: cargo xtask release client build-raw-specs $NETWORK_TAG"
    cargo xtask release client build-raw-specs "$NETWORK_TAG"
    ok "Raw specs built."

    # Sub-step: create client release
    info "Running: cargo xtask release client create $NETWORK_TAG $NETWORK_BRANCH"
    cargo xtask release client create "$NETWORK_TAG" "$NETWORK_BRANCH"
    ok "Client release created."

    # Sub-step: trigger CI builds
    echo ""
    warn "The next command triggers CI builds which take ~30-90 minutes."
    echo "  This will build: DEB/RPM (x64+ARM), Docker images (amd64+arm64)"
    echo "  Docker image: ${DOCKER_IMAGE}:${DOCKER_TAG}"
    echo ""
    confirm_or_die "Trigger CI builds?"

    info "Running: cargo xtask release client trigger-builds $NETWORK_TAG $NETWORK_BRANCH"
    cargo xtask release client trigger-builds "$NETWORK_TAG" "$NETWORK_BRANCH"
    ok "CI builds triggered. Docker image will be: ${DOCKER_IMAGE}:${DOCKER_TAG}"
  fi

  echo ""
  warn "Wait for CI to complete before proceeding to step 7."
  echo "  Monitor at: https://git.duniter.org/nodes/rust/duniter-v2s/-/pipelines"
  echo ""
  confirm_or_die "CI builds completed? Ready to deploy bootstrap node?"
}

step7_deploy_bootstrap_node() {
  step_header 7 "Deploy bootstrap node"

  local dns_host node_name node_key_path session_suri

  # --- Auto-detect the correct secrets file ---
  # The secrets file must match the session keys in the chain spec (g1.yaml).
  # When setup_bootstrap_keys runs multiple times, only the keys matching the
  # built chain spec are valid. We verify by comparing the SS58 babe address.
  local genesis_babe_address=""
  local chain_spec="release/network/${RUNTIME}.json"
  if [ -f "$chain_spec" ]; then
    # Extract the babe key of the authority with distinct keys (the bootstrap smith)
    genesis_babe_address=$(python3 -c "
import json,sys
spec = json.load(open('$chain_spec'))
keys_list = spec['genesis']['runtimeGenesis']['patch']['session']['keys']
for entry in keys_list:
    keys = entry[2] if len(entry) > 2 else entry[1]
    vals = [keys.get(k,'') for k in ['grandpa','babe','im_online','authority_discovery']]
    if len(set(vals)) > 1:
        print(keys['babe'])
        break
" 2>/dev/null)
    if [ -n "$genesis_babe_address" ]; then
      info "Genesis babe authority: $genesis_babe_address"
    fi
  fi

  if [ -z "$SECRETS_FILE" ] || [ ! -f "$SECRETS_FILE" ]; then
    # Find the secrets file whose SS58 Address matches the genesis babe key
    local found_match=""
    for sf in $(ls -t launch-secrets-${NETWORK_TAG}-*.txt 2>/dev/null); do
      local sf_ss58
      sf_ss58=$(grep 'SS58 Address:' "$sf" 2>/dev/null | head -1 | sed 's/.*SS58 Address: *//' || true)
      if [ -n "$genesis_babe_address" ] && [ -n "$sf_ss58" ] && [ "$sf_ss58" = "$genesis_babe_address" ]; then
        SECRETS_FILE="$sf"
        found_match="yes"
        ok "Secrets file matches genesis chain spec: $SECRETS_FILE"
        break
      fi
    done
    if [ -z "$found_match" ]; then
      # Fallback to latest if no chain spec to compare
      local latest_secrets
      latest_secrets=$(ls -t launch-secrets-${NETWORK_TAG}-*.txt 2>/dev/null | head -1)
      if [ -n "$latest_secrets" ]; then
        SECRETS_FILE="$latest_secrets"
        warn "Using latest secrets file (could not verify against chain spec): $SECRETS_FILE"
      fi
    fi
  fi

  # --- Node key: auto-use from deploy-bootstrap/ ---
  local default_node_key="${DEPLOY_DIR}/node.key"
  if [ -f "$default_node_key" ]; then
    node_key_path="$default_node_key"
    ok "Using node.key: $node_key_path"
  else
    node_key_path=$(prompt_with_default "Path to node.key (or 'generate' for auto)" "generate")
  fi

  # --- Session keys: auto-read mnemonic from secrets file ---
  if [ -n "$SECRETS_FILE" ] && [ -f "$SECRETS_FILE" ]; then
    session_suri=$(grep 'Secret phrase:' "$SECRETS_FILE" | head -1 | sed 's/.*Secret phrase: *//')
  fi
  if [ -z "$session_suri" ]; then
    warn "Could not auto-read session key mnemonic from secrets file."
    session_suri=$(prompt_secret "Session keys secret phrase (from checklist A4 / secrets file)")
  else
    ok "Session key mnemonic auto-loaded from secrets file"
  fi
  [ -n "$session_suri" ] || die "Session key mnemonic is empty. Cannot deploy bootstrap node."

  # --- Remaining prompts ---
  dns_host=$(prompt_with_default "DNS hostname of bootstrap node" "g1-bootstrap.p2p.legal")
  node_name=$(prompt_with_default "Node name" "${RUNTIME}-bootstrap")

  # Create deploy directory
  mkdir -p "$DEPLOY_DIR"

  # Handle node key
  if [ "$node_key_path" = "generate" ]; then
    info "Node key will be auto-generated by Docker entrypoint."
    warn "For production, generate a node key in advance (checklist A3) so Peer ID matches bootNodes."
  elif [ -f "$node_key_path" ]; then
    ok "node.key ready in ${DEPLOY_DIR}/"
  else
    die "Node key file not found: $node_key_path"
  fi

  # Generate docker-compose.yml
  local compose_file="${DEPLOY_DIR}/docker-compose.yml"
  info "Generating ${compose_file}..."

  local node_key_volume=""
  if [ -f "${DEPLOY_DIR}/node.key" ]; then
    node_key_volume="      - ./node.key:/var/lib/duniter/node.key:ro"
  fi

  cat > "$compose_file" <<COMPOSE_EOF
services:
  duniter-${RUNTIME}-smith:
    image: ${DOCKER_IMAGE}:${DOCKER_TAG}
    restart: unless-stopped
    ports:
      - 127.0.0.1:9944:9944
      - 30333:30333
    environment:
      DUNITER_NODE_NAME: "${node_name}"
      DUNITER_CHAIN_NAME: "${RUNTIME}"
      DUNITER_VALIDATOR: "true"
      DUNITER_PRUNING_PROFILE: light
      DUNITER_PUBLIC_ADDR: /dns/${dns_host}/tcp/30333
      DUNITER_LISTEN_ADDR: /ip4/0.0.0.0/tcp/30333
    volumes:
      - ${RUNTIME}-data:/var/lib/duniter
${node_key_volume}

  distance-oracle:
    image: ${DOCKER_IMAGE}:${DOCKER_TAG}
    restart: unless-stopped
    entrypoint: docker-distance-entrypoint
    environment:
      ORACLE_RPC_URL: ws://duniter-${RUNTIME}-smith:9944
      ORACLE_RESULT_DIR: /var/lib/duniter/chains/${RUNTIME}/distance/
      ORACLE_EXECUTION_INTERVAL: 1800
    volumes:
      - ${RUNTIME}-data:/var/lib/duniter

volumes:
  ${RUNTIME}-data:
COMPOSE_EOF

  ok "docker-compose.yml generated."

  _deploy_bootstrap_local "$session_suri" "$dns_host"

  # Clear sensitive variable
  session_suri=""
}

_deploy_bootstrap_local() {
  local session_suri="$1"
  local dns_host="$2"

  info "Pulling Docker image: ${DOCKER_IMAGE}:${DOCKER_TAG}"
  docker pull "${DOCKER_IMAGE}:${DOCKER_TAG}"

  # Inject session keys before first start
  info "Injecting session keys..."
  docker compose -f "${DEPLOY_DIR}/docker-compose.yml" run --rm \
    "duniter-${RUNTIME}-smith" -- key generate-session-keys \
    --chain "$RUNTIME" -d /var/lib/duniter --suri "$session_suri"
  ok "Session keys injected."

  # Start in foreground first to verify
  info "Starting node in foreground (Ctrl+C to stop after verification)..."
  echo ""
  warn "Watch for:"
  echo "  - 'Local node identity is: 12D3KooW...' (should match bootNodes)"
  echo "  - 'Prepared block for proposing' messages"
  echo ""
  confirm_or_die "Start node in foreground?"

  # Start node in background
  info "Starting node in background..."
  docker compose -f "${DEPLOY_DIR}/docker-compose.yml" up -d
  ok "Node started. Waiting for block production..."

  # Poll RPC to verify block production
  info "Polling RPC every 6s (timeout 120s)..."
  local elapsed=0
  local block_num=0
  while [ "$elapsed" -lt 120 ]; do
    sleep 6
    elapsed=$((elapsed + 6))

    local result
    result=$(curl -s -m 5 \
      -H "Content-Type: application/json" \
      -d '{"id":1,"jsonrpc":"2.0","method":"chain_getHeader","params":[]}' \
      http://127.0.0.1:9944 2>/dev/null || echo "")

    if [ -n "$result" ]; then
      local hex_num
      hex_num=$(echo "$result" | jq -r '.result.number // empty' 2>/dev/null || echo "")
      if [ -n "$hex_num" ]; then
        block_num=$((hex_num))
        info "Current block: $block_num"
        if [ "$block_num" -ge 2 ]; then
          ok "Block $block_num reached! Node is producing blocks."
          break
        fi
      fi
    fi
  done

  if [ "$block_num" -lt 2 ]; then
    warn "Block 2 not reached within 120s."
    echo "  Check logs: docker compose -f ${DEPLOY_DIR}/docker-compose.yml logs -f"
    confirm_or_die "Continue anyway?"
  fi

  echo ""
  info "Verify Peer ID matches bootNodes:"
  echo "  docker compose -f ${DEPLOY_DIR}/docker-compose.yml logs duniter-${RUNTIME}-smith | grep 'Local node identity'"
  echo ""
}


step8_session_key_rotation() {
  step_header 8 "Session key rotation (optional)"

  echo ""
  info "This step rotates session keys on the running node."
  info "The genesis keys (from the build machine) will be replaced by keys"
  info "generated directly on the server."
  echo ""

  local do_rotate
  do_rotate=$(prompt_with_default "Rotate session keys now? (yes/no)" "no")

  if [ "$do_rotate" != "yes" ]; then
    info "Skipping session key rotation."
    return 0
  fi

  local rpc_url
  rpc_url=$(prompt_with_default "RPC endpoint" "http://127.0.0.1:9944")

  info "Calling author_rotateKeys..."
  local result
  result=$(curl -s -H "Content-Type: application/json" \
    -d '{"id":1,"jsonrpc":"2.0","method":"author_rotateKeys","params":[]}' \
    "$rpc_url")

  local new_keys
  new_keys=$(echo "$result" | jq -r '.result // empty')

  if [ -z "$new_keys" ]; then
    warn "Failed to rotate keys. Response: $result"
    warn "Ensure RPC is accessible and --rpc-methods=Unsafe is set."
    return 0
  fi

  ok "New session keys: $new_keys"
  echo ""
  warn "ACTION REQUIRED: Submit these keys on-chain via session.setKeys"
  echo "  Use polkadot.js/apps or subxt to call session.setKeys("
  echo "    keys: $new_keys,"
  echo "    proof: 0x"
  echo "  )"
  echo "  New keys take effect after one epoch (~4h)."
  echo ""
  confirm_or_die "session.setKeys submitted on-chain?"
}

step9_additional_smiths() {
  step_header 9 "Additional smiths"

  echo ""
  info "Each additional smith needs to:"
  echo ""
  echo "  1. Set up a server (4 GB RAM, 2 CPU, 100 GB SSD)"
  echo ""
  echo "  2. Use this docker-compose.yml (adapt name, DNS, bootnode):"
  cat <<SMITH_EOF

    services:
      duniter-${RUNTIME}-smith:
        image: ${DOCKER_IMAGE}:${DOCKER_TAG}
        restart: unless-stopped
        ports:
          - 127.0.0.1:9944:9944
          - 30333:30333
        environment:
          DUNITER_NODE_NAME: "<smith-name>"
          DUNITER_CHAIN_NAME: "${RUNTIME}"
          DUNITER_VALIDATOR: "true"
          DUNITER_PRUNING_PROFILE: light
          DUNITER_PUBLIC_ADDR: /dns/<smith-dns>/tcp/30333
          DUNITER_LISTEN_ADDR: /ip4/0.0.0.0/tcp/30333
        volumes:
          - ${RUNTIME}-data:/var/lib/duniter
          - ./node.key:/var/lib/duniter/node.key:ro

      distance-oracle:
        image: ${DOCKER_IMAGE}:${DOCKER_TAG}
        restart: unless-stopped
        entrypoint: docker-distance-entrypoint
        environment:
          ORACLE_RPC_URL: ws://duniter-${RUNTIME}-smith:9944
          ORACLE_RESULT_DIR: /var/lib/duniter/chains/${RUNTIME}/distance/
          ORACLE_EXECUTION_INTERVAL: 1800
        volumes:
          - ${RUNTIME}-data:/var/lib/duniter

    volumes:
      ${RUNTIME}-data:

SMITH_EOF
  echo ""
  echo "  3. Inject session keys:"
  echo "     docker compose run --rm duniter-${RUNTIME}-smith -- key generate-session-keys \\"
  echo "       --chain ${RUNTIME} -d /var/lib/duniter --suri '<secret phrase>'"
  echo ""
  echo "  4. Start: docker compose up -d"
  echo ""
  echo "  5. Rotate keys and submit session.setKeys on-chain"
  echo ""

  confirm_or_die "Additional smiths deployed (or skipping for now)?"
}

step10_mirror_nodes() {
  step_header 10 "Mirror nodes"

  echo ""
  info "Mirror node docker-compose template:"
  cat <<MIRROR_EOF

    services:
      duniter-${RUNTIME}-mirror:
        image: ${DOCKER_IMAGE}:${DOCKER_TAG}
        restart: unless-stopped
        ports:
          - "9944:9944"
          - "30333:30333"
        environment:
          DUNITER_NODE_NAME: "${RUNTIME}-mirror"
          DUNITER_CHAIN_NAME: "${RUNTIME}"
          DUNITER_PRUNING_PROFILE: archive
          DUNITER_PUBLIC_RPC: "wss://rpc.${RUNTIME}.duniter.org"
        volumes:
          - ${RUNTIME}-mirror:/var/lib/duniter

    volumes:
      ${RUNTIME}-mirror:

MIRROR_EOF
  echo ""
  info "Nginx reverse proxy for WSS:"
  cat <<NGINX_EOF

    server {
        listen 443 ssl;
        server_name rpc.${RUNTIME}.duniter.org;
        location / {
            proxy_pass http://127.0.0.1:9944;
            proxy_http_version 1.1;
            proxy_set_header Upgrade \$http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_read_timeout 86400;
        }
    }

NGINX_EOF
  echo ""
  warn "A mirror RPC archive node must be running before step 11 (squid)."
  confirm_or_die "At least one mirror RPC archive node is ready?"
}

step11_squid_trigger() {
  step_header 11 "Squid indexer builds"

  local default_rpc="wss://${RUNTIME}.p2p.legal/ws"
  local rpc_url
  rpc_url=$(prompt_with_default "WSS URL of RPC archive node" "$default_rpc")

  local squid_args=("$NETWORK_TAG")
  if [ "$rpc_url" != "$default_rpc" ]; then
    squid_args+=(--rpc-url "$rpc_url")
  fi

  info "Running: cargo xtask release squid trigger-builds ${squid_args[*]}"
  confirm_or_die "Trigger squid CI builds?"

  cargo xtask release squid trigger-builds "${squid_args[@]}"

  ok "Squid CI builds triggered."
  echo ""
  info "This will build Docker images:"
  echo "  - duniter/squid-app-${RUNTIME}:<version>"
  echo "  - duniter/squid-graphile-${RUNTIME}:<version>"
  echo "  - duniter/squid-postgres-${RUNTIME}:<version>"
}

# ============================================================================
# 5. Post-launch verifications
# ============================================================================

verify_docker_hub_image() {
  info "Checking Docker Hub image: ${DOCKER_IMAGE}:${DOCKER_TAG}..."

  local token
  token=$(curl -s "https://auth.docker.io/token?service=registry.docker.io&scope=repository:${DOCKER_IMAGE}:pull" | jq -r '.token // empty')

  if [ -z "$token" ]; then
    warn "Could not get Docker Hub auth token."
    return 1
  fi

  local http_code
  http_code=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "Authorization: Bearer ${token}" \
    -H "Accept: application/vnd.docker.distribution.manifest.v2+json" \
    "https://registry-1.docker.io/v2/${DOCKER_IMAGE}/manifests/${DOCKER_TAG}")

  if [ "$http_code" = "200" ] || [ "$http_code" = "302" ]; then
    ok "Docker Hub image exists."
    return 0
  else
    warn "Docker Hub image not found (HTTP $http_code)."
    return 1
  fi
}

verify_gitlab_release() {
  info "Checking GitLab release: ${NETWORK_TAG}..."

  local result
  result=$(curl -s \
    -H "PRIVATE-TOKEN: ${GITLAB_TOKEN}" \
    "https://git.duniter.org/api/v4/projects/${GITLAB_PROJECT_ID}/releases/${NETWORK_TAG}")

  local tag
  tag=$(echo "$result" | jq -r '.tag_name // empty')

  if [ -n "$tag" ]; then
    local asset_count
    asset_count=$(echo "$result" | jq '.assets.links | length')
    ok "GitLab release found: tag=$tag, assets=$asset_count"
    return 0
  else
    warn "GitLab release '${NETWORK_TAG}' not found."
    return 1
  fi
}

verify_block_production() {
  local rpc_url="${1:-http://127.0.0.1:9944}"
  info "Checking block production on ${rpc_url}..."

  local result
  result=$(curl -s -m 10 \
    -H "Content-Type: application/json" \
    -d '{"id":1,"jsonrpc":"2.0","method":"chain_getHeader","params":[]}' \
    "$rpc_url" 2>/dev/null || echo "")

  if [ -z "$result" ]; then
    warn "Could not connect to RPC at ${rpc_url}."
    return 1
  fi

  local hex_num
  hex_num=$(echo "$result" | jq -r '.result.number // empty' 2>/dev/null || echo "")

  if [ -n "$hex_num" ]; then
    local block_num=$((hex_num))
    ok "Current block: $block_num"
    return 0
  else
    warn "Could not read block number from RPC response."
    return 1
  fi
}

verify_finalization() {
  local rpc_url="${1:-http://127.0.0.1:9944}"
  info "Checking GRANDPA finalization on ${rpc_url}..."

  local result
  result=$(curl -s -m 10 \
    -H "Content-Type: application/json" \
    -d '{"id":1,"jsonrpc":"2.0","method":"chain_getFinalizedHead","params":[]}' \
    "$rpc_url" 2>/dev/null || echo "")

  if [ -z "$result" ]; then
    warn "Could not connect to RPC at ${rpc_url}."
    return 1
  fi

  local finalized_hash
  finalized_hash=$(echo "$result" | jq -r '.result // empty' 2>/dev/null || echo "")

  if [ -n "$finalized_hash" ] && [ "$finalized_hash" != "null" ]; then
    ok "Finalized head: ${finalized_hash:0:16}..."
    return 0
  else
    warn "No finalized head found (GRANDPA may need 2/3 of validators online)."
    return 1
  fi
}

run_post_launch_checks() {
  echo ""
  echo -e "${BOLD}${CYAN}======================================${NC}"
  echo -e "${BOLD}${CYAN} Post-launch verifications${NC}"
  echo -e "${BOLD}${CYAN}======================================${NC}"
  echo ""

  local failures=0

  verify_gitlab_release  || ((failures++))
  verify_docker_hub_image || ((failures++))

  local rpc_url
  rpc_url=$(prompt_with_default "RPC endpoint for block checks" "http://127.0.0.1:9944")
  verify_block_production "$rpc_url" || ((failures++))
  verify_finalization "$rpc_url" || ((failures++))

  echo ""
  if [ "$failures" -eq 0 ]; then
    ok "All post-launch checks passed!"
  else
    warn "$failures check(s) failed. Review warnings above."
  fi

  echo ""
  info "Manual checks remaining:"
  echo "  [ ] Smiths connected and online"
  echo "  [ ] Distance oracle operational"
  echo "  [ ] Identities and certifications migrated"
  echo "  [ ] Correct monetary mass"
  echo "  [ ] UD created after 24h"
  echo "  [ ] Public RPC accessible"
  echo "  [ ] Squid Docker images published on Docker Hub"
}

# ============================================================================
# 6. Error handling and cleanup
# ============================================================================

cleanup() {
  local exit_code=$?

  # Clear sensitive variables
  DUNITERTEAM_PASSWD=""
  GITLAB_TOKEN=""

  if [ "$exit_code" -ne 0 ] && [ "$CURRENT_STEP" -gt 0 ]; then
    echo ""
    echo -e "${RED}======================================${NC}"
    echo -e "${RED} Launch failed at step ${CURRENT_STEP}/${TOTAL_STEPS}${NC}"
    echo -e "${RED}======================================${NC}"
    echo ""
    echo "To resume from this step:"
    echo "  ./scripts/launch-network.sh --runtime ${RUNTIME} --resume-from ${CURRENT_STEP}"
    echo ""
  fi
}

trap cleanup EXIT

# ============================================================================
# 7. Main
# ============================================================================

usage() {
  cat <<EOF
Usage: ./scripts/launch-network.sh [OPTIONS]

Automated launch of a Duniter v2s network.

Options:
  --runtime NAME     Runtime: g1|gtest|gdev (default: g1)
  --resume-from N    Resume from step N (1-11)
  --skip-build       Skip long builds (assumes release/ is populated)
  --dump-url URL     Custom G1 v1 dump URL
  -h, --help         Show this help

Required environment variables:
  GITLAB_TOKEN         GitLab Access Token (scope: api)
  DUNITERTEAM_PASSWD   Docker Hub password for org duniter

Steps:
   1. Create network branch
   2. G1 migration data
   3. Build WASM runtime
   4. Build network specs
   5. Network release (GitLab)
   6. Client release
   7. Deploy bootstrap node
   8. Session key rotation (optional)
   9. Additional smiths (informational)
  10. Mirror nodes (informational)
  11. Squid indexer builds
EOF
}

main() {
  # Defaults
  RUNTIME="g1"
  RESUME_FROM=1
  SKIP_BUILD="false"
  DUMP_URL=""

  # Parse arguments
  while [ $# -gt 0 ]; do
    case "$1" in
      --runtime)
        RUNTIME="$2"; shift 2
        ;;
      --resume-from)
        RESUME_FROM="$2"; shift 2
        ;;
      --skip-build)
        SKIP_BUILD="true"; shift
        ;;
      --dump-url)
        DUMP_URL="$2"; shift 2
        ;;
      -h|--help)
        usage; exit 0
        ;;
      *)
        die "Unknown option: $1. Use --help for usage."
        ;;
    esac
  done

  # Validate runtime
  case "$RUNTIME" in
    g1|gtest|gdev) ;;
    *) die "Invalid runtime: $RUNTIME. Must be g1, gtest, or gdev." ;;
  esac

  # Validate resume-from
  if [ "$RESUME_FROM" -lt 1 ] || [ "$RESUME_FROM" -gt "$TOTAL_STEPS" ]; then
    die "--resume-from must be between 1 and $TOTAL_STEPS."
  fi

  # Detect platform → DATE_CMD (prefer gdate on macOS for full GNU compat, fallback to date)
  if command -v gdate &>/dev/null; then
    DATE_CMD="gdate"
  else
    DATE_CMD="date"
  fi

  echo ""
  echo -e "${BOLD}${CYAN}============================================${NC}"
  echo -e "${BOLD}${CYAN}  Duniter v2s Network Launch Script${NC}"
  echo -e "${BOLD}${CYAN}============================================${NC}"
  echo ""

  # Detect versions
  info "Detecting versions..."
  detect_spec_version
  detect_client_version
  derive_names
  resolve_dump_url

  # Run validations (skip if resuming from a late step)
  if [ "$RESUME_FROM" -le 1 ]; then
    info "Running full prerequisite validation..."
    validate_prerequisites
    validate_g1_yaml
    validate_client_specs
    setup_bootstrap_keys
  else
    info "Resuming from step $RESUME_FROM — running minimal validation..."
    require_env GITLAB_TOKEN "Export GITLAB_TOKEN with scope 'api'."
    require_env DUNITERTEAM_PASSWD "Export DUNITERTEAM_PASSWD (Docker Hub org duniter)."
  fi

  # Show summary and confirm
  local yaml_file="resources/${RUNTIME}.yaml"
  echo ""
  echo -e "${BOLD}Configuration summary:${NC}"
  echo "  Runtime:          $RUNTIME"
  echo "  spec_version:     $SPEC_VERSION  (runtime/${RUNTIME}/src/lib.rs)"
  echo "  client_version:   $CLIENT_VERSION  (node/Cargo.toml)"
  echo "  Network tag:      $NETWORK_TAG"
  echo "  Network branch:   $NETWORK_BRANCH"
  echo "  Docker image:     $DOCKER_IMAGE:$DOCKER_TAG"
  echo "  Starting step:    $RESUME_FROM"
  if [ "$SKIP_BUILD" = "true" ]; then
    echo "  Skip builds:      YES"
  fi
  if [ -n "$DUMP_URL" ]; then
    echo "  G1v1 dump URL:    $RESOLVED_DUMP_URL (custom)"
  else
    echo "  G1v1 dump URL:    $RESOLVED_DUMP_URL (auto)"
  fi

  # Display initial smiths
  echo ""
  echo -e "${BOLD}Initial smiths${NC}  (${yaml_file})${BOLD}:${NC}"
  local smith_names
  smith_names=$(yq_raw '.smiths | keys | .[]' "$yaml_file")
  local bootstrap_smith=""
  while IFS= read -r name; do
    local has_keys
    has_keys=$(yq_raw ".smiths.\"${name}\".session_keys // \"\"" "$yaml_file")
    if [ -n "$has_keys" ]; then
      echo -e "  - ${name} ${GREEN}(bootstrap)${NC}"
      bootstrap_smith="$name"
    else
      echo "  - ${name}"
    fi
  done <<< "$smith_names"
  echo "  Total: $(echo "$smith_names" | wc -l | tr -d ' ') smiths"

  # Display technical committee
  echo ""
  echo -e "${BOLD}Technical committee${NC}  (${yaml_file})${BOLD}:${NC}"
  local tc_members
  tc_members=$(yq_raw '.technical_committee[]' "$yaml_file")
  while IFS= read -r member; do
    echo "  - ${member}"
  done <<< "$tc_members"
  echo "  Total: $(echo "$tc_members" | wc -l | tr -d ' ') members"

  echo ""
  confirm_or_die "Proceed with launch?"

  # Execute steps
  [ "$RESUME_FROM" -le 1 ]  && step1_create_network_branch
  [ "$RESUME_FROM" -le 2 ]  && step2_g1_migration_data
  [ "$RESUME_FROM" -le 3 ]  && step3_build_wasm_runtime
  [ "$RESUME_FROM" -le 4 ]  && step4_build_network_specs
  [ "$RESUME_FROM" -le 5 ]  && step5_network_release
  [ "$RESUME_FROM" -le 6 ]  && step6_client_release
  [ "$RESUME_FROM" -le 7 ]  && step7_deploy_bootstrap_node
  [ "$RESUME_FROM" -le 8 ]  && step8_session_key_rotation
  [ "$RESUME_FROM" -le 9 ]  && step9_additional_smiths
  [ "$RESUME_FROM" -le 10 ] && step10_mirror_nodes
  [ "$RESUME_FROM" -le 11 ] && step11_squid_trigger

  # Post-launch checks
  run_post_launch_checks

  echo ""
  echo -e "${BOLD}${GREEN}============================================${NC}"
  echo -e "${BOLD}${GREEN}  Network launch complete!${NC}"
  echo -e "${BOLD}${GREEN}============================================${NC}"
  echo ""
}

main "$@"
