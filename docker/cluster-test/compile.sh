#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

. /root/.cargo/env
cargo build -p cluster-test --target-dir /target --release
