#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Launch a 4 node libra swarm and run cluster test against it for a short period

set -e

RUST_BACKTRACE=1 cargo run -p libra-swarm -- -n 4 -l > swarm.out &
while [ true ]; do
    sleep 5;
    if [ ! -z "$(cat swarm.out | grep cluster-test | xargs )" ]; then
        break
    fi
done
ls -l swarm.out
CMD=$(cat swarm.out | grep cluster-test | xargs)
eval "RUST_BACKTRACE=1 $CMD --duration 90"
kill %1
rm -f swarm.out
