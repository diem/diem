#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
PROXY_RUN=""
if [ "$https_proxy" ]; then
    PROXY=" --build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
    PROXY_RUN=" --env https_proxy=$https_proxy --env http_proxy=$http_proxy"
fi

rm -f libra-node-from-docker

DOCKERFILE="$(mktemp -t Dockerfile.XXXXXX)"
awk '/FROM.*AS builder/ { exit } { print } ' $DIR/../validator/validator.Dockerfile > "$DOCKERFILE"
docker build -f "$DOCKERFILE" $DIR/../.. --tag libra-node-builder $PROXY

TARGET=$DIR/../../target/libra-node-builder/
mkdir -p $TARGET
if docker ps -f name=libra-node-builder-container -f ancestor=libra-node-builder | grep libra-node-builder-container ; then
  echo "Builder container is running with latest builder image"
else
  echo "Builder image has changed, re-creating builder container"
  docker rm -f libra-node-builder-container || echo "Build container did not exist"
  docker run --name libra-node-builder-container -d -i -t -v $DIR/../..:/libra $PROXY_RUN libra-node-builder bash
fi

docker exec -i -t libra-node-builder-container bash -c 'source /root/.cargo/env; cargo build --release -p libra-node --target-dir /target && /bin/cp /target/release/libra-node target/libra-node-builder/'

DOCKERFILE="$(mktemp -t Dockerfile.XXXXXX)"
awk '/^FROM.*AS prod/ { from=1 } /^COPY/ { sub(/--from=.*release/, "target/libra-node-builder", $0) } { if (from) print }' $DIR/../validator/validator.Dockerfile > "$DOCKERFILE"
docker build -f "$DOCKERFILE" $DIR/../.. --tag libra_e2e --build-arg GIT_REV="$(git rev-parse HEAD)" --build-arg GIT_UPSTREAM="$(git merge-base HEAD origin/master)" --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" $PROXY
