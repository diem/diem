#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

./build.sh dev -n 4
./build.sh 100 -n 100
