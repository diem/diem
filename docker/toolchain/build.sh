#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
if [ "$https_proxy" ]; then
    PROXY=" --build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
fi

docker build -f $DIR/Dockerfile \
  $DIR/../.. \
  --tag toolchain_rust:$(cat $DIR/../../rust-toolchain)-$(shasum -a 256 $DIR/../../Cargo.lock | colrm 9)
