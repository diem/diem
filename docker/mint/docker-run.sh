#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -ex
export RUST_BACKTRACE=full
/opt/libra/bin/config-builder faucet \
    -o /opt/libra/etc \
    -s $CFG_SEED

cd /opt/libra/bin && \
exec gunicorn --bind 0.0.0.0:8000 --access-logfile - --error-logfile - --log-level $LOG_LEVEL server
