#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
if [ "$https_proxy" ]; then
  PROXY=" --build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
fi

rm -f cluster_test_docker_builder_cluster_test

docker build -f $DIR/cluster_test_builder.Dockerfile $DIR/../.. --tag libra_cluster_test_builder $PROXY
TARGET=$DIR/../../target/cluster_test_docker_builder/
mkdir -p $TARGET
if docker ps -f name=libra_cluster_test_builder_container -f ancestor=libra_cluster_test_builder | grep libra_cluster_test_builder_container ; then
  echo "Builder container is running with latest builder image"
else
  echo "Builder image has changed, re-creating builder container"
  docker rm -f libra_cluster_test_builder_container || echo "Build container did not exist"
  docker run --name libra_cluster_test_builder_container -d -i -t -v $DIR/../..:/libra libra_cluster_test_builder bash
fi

docker exec -i -t libra_cluster_test_builder_container bash -c 'source /root/.cargo/env; export GIT_REV=""; cargo build -p cluster-test --target-dir /target && /bin/cp /target/debug/cluster-test target/cluster_test_docker_builder/'

if [ "$1" = "--build-docker-image" ]; then
  ln target/cluster_test_docker_builder/cluster_test cluster_test_docker_builder_cluster_test
  docker build -f $DIR/cluster_test.Dockerfile $DIR/../.. --tag libra_cluster_test --build-arg GIT_REV="$(git rev-parse HEAD)" --build-arg GIT_UPSTREAM="$(git merge-base HEAD origin/master)" --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" $PROXY
  rm cluster_test_docker_builder_cluster_test
else
  echo "Not building docker container, run with --build-docker-image to build docker image"
  echo "Built binary is in target/cluster_test_docker_builder/cluster-test"
fi
