#!/bin/sh
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"

export WAYPOINT=$(cat "$DIR"/validator_data/waypoint.txt)
echo "Waypoint: $WAYPOINT"

docker run --network diem-docker-compose-shared \
          -it libra/client:devnet \
          --url http://172.16.1.3:8080 \
          --faucet-url http://172.16.1.2:8000 \
          --chain-id TESTING \
          --waypoint "$WAYPOINT"
