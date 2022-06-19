#!/usr/bin/env bash

RN_LABELS=(
  RN-binary
  RN-runtime
  RN-silent
)

MR_LABELS=$1
MR_MILESTONE=$2

echo "Labels: $MR_LABELS"
echo "Milestone: $MR_MILESTONE"

if [[ "$MR_LABELS" == *"RN-runtime"* ]]; then
  if [[ "$MR_MILESTONE" != "runtime-"* ]]; then
    echo "MR with runtime changes should have a runtime-* milestone."
    exit 1
  fi
fi

for RN_LABEL in ${RN_LABELS[@]}; do
  if [[ "$MR_LABELS" == *"$RN_LABEL"* ]]; then
    exit 0
  fi
done

echo "Every MR should have at least one RN-* label."
exit 1
