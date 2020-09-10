#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

source "${DIR}/ct.vars"

for ((i = 0; i < ${K8S_POOL_SIZE}; i++)); do
  ws="ct-${i}"
  echo "Workspace: $ws"
  echo "Cluster test pods:"
  context="arn:aws:eks:us-west-2:853397791086:cluster/${ws}-k8s-testnet"
  kubectl get pods --context="${context}" -l app=cluster-test --sort-by=.status.startTime
  echo
done
