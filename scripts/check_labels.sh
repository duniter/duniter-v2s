#!/usr/bin/env bash

RN_LABELS=(
  RN-binary
  RN-runtime
  RN-silent
)

for RN_LABEL in ${RN_LABELS[@]}; do
  if [[ "$1" == *"$RN_LABEL"* ]]; then
    echo "exit 0"
    exit 0
  fi
done

echo "exit 1"
exit 1
