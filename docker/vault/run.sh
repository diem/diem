#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

docker run --cap-add=IPC_LOCK --publish 8200:8200 --detach vault
