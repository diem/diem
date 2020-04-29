#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -ex
export RUST_BACKTRACE=full

declare -a params
if [ -n "${CFG_SEED}" ]; then
    params+="-s $CFG_SEED "
fi

if [ -n "${CFG_NUM_VALIDATORS}" ]; then # Total number of nodes in this network
	  params+="-n ${CFG_NUM_VALIDATORS} "
fi

/opt/libra/bin/config-builder faucet \
    -o /opt/libra/etc \
    ${params[@]}

cd /opt/libra/bin && \
exec gunicorn --bind 0.0.0.0:8000 --access-logfile - --error-logfile - --log-level $LOG_LEVEL server
