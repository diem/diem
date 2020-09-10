#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

cd /opt/libra/etc
[ -n "${NODE_CONFIG}" ] && echo "$NODE_CONFIG" > node.yaml
exec /opt/libra/bin/libra-node -f node.yaml
