#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
if [ "$https_proxy" ]; then
    PROXY=" --build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
fi

TOOLCHAIN_TAG=$(cat $DIR/../../rust-toolchain)-$(shasum -a 256 $DIR/../../Cargo.lock | colrm 9)
if ! docker image inspect toolchain_rust:$TOOLCHAIN_TAG &> /dev/null; then
  echo "Image toolchain_rust:$TOOLCHAIN_TAG does not exist locally. Will build it."
  $DIR/../toolchain/build.sh
fi

docker build -f $DIR/Dockerfile \
  $DIR/../.. \
  --tag libra_client \
  --build-arg TOOLCHAIN_TAG="$TOOLCHAIN_TAG" \
  --build-arg GIT_REV="$(git rev-parse HEAD)" \
  --build-arg GIT_UPSTREAM="$(git merge-base HEAD origin/master)" \
  --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
  $PROXY
