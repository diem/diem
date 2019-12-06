#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -ex
export RUST_BACKTRACE=full
/opt/libra/bin/dynamic-config-builder -d /opt/libra/etc -o /opt/libra/etc --faucet-client -a "/ip4/0.0.0.0/tcp/1" \
-b "/ip4/0.0.0.0/tcp/1" -l "/ip4/0.0.0.0/tcp/1" -n $CFG_NUM_VALIDATORS -s $CFG_SEED

cd /opt/libra/bin && \
exec gunicorn --bind 0.0.0.0:8000 --access-logfile - --error-logfile - --log-level $LOG_LEVEL server
