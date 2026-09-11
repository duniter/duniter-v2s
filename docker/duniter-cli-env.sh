#!/bin/bash
# Values are passed as arguments, never evaluated as shell code.
duniter_has_arg() {
  local flag=$1 short=${2:-} arg
  # Set by the entrypoint before loading environment options.
  # shellcheck disable=SC2154
  for arg in "${cli_args[@]}"; do
    [[ $arg != -- ]] || break
    case "$arg" in
      "--$flag"|"--$flag="*) return 0 ;;
    esac
    if [[ -n $short && $arg == "-$short"* ]]; then return 0; fi
  done
  return 1
}

duniter_env_option() {
  local name=$1 flag=$2 action=$3 short=$4 value=${!1:-} item i
  [[ -n $value ]] || return 0
  duniter_has_arg "$flag" "$short" && return 0
  case "$action" in
    set) env_args+=("--$flag=$value") ;;
    set_true|set_false)
      case "$value" in
        [Tt][Rr][Uu][Ee]|[Yy][Ee][Ss]|1) env_args+=("--$flag") ;;
        [Ff][Aa][Ll][Ss][Ee]|[Nn][Oo]|0) ;;
        *) echo "$name must be true/false, yes/no, or 1/0" >&2; return 1 ;;
      esac ;;
    append)
      while IFS= read -r item || [[ -n $item ]]; do
        [[ -n $item ]] || { echo "$name contains an empty list item" >&2; return 1; }
        env_args+=("--$flag=$item")
      done <<<"$value" ;;
    count)
      if [[ ! $value =~ ^[0-9]{1,3}$ ]] || (( 10#$value > 255 )); then
        echo "$name must be an integer between 0 and 255" >&2; return 1
      fi
      for ((i=0; i<10#$value; i++)); do env_args+=("--$flag"); done ;;
    *) echo "Unsupported CLI action: $action" >&2; return 1 ;;
  esac
}
