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

docker build -f $DIR/cluster-test.builder.Dockerfile $DIR/../.. --tag libra_cluster_test_builder $PROXY
TARGET=$DIR/../../target/cluster_test_docker_builder/
mkdir -p $TARGET
if docker ps -f name=libra_cluster_test_builder_container -f ancestor=libra_cluster_test_builder | grep libra_cluster_test_builder_container ; then
  echo "Builder container is running with latest builder image"
else
  echo "Builder image has changed, re-creating builder container"
  docker rm -f libra_cluster_test_builder_container || echo "Build container did not exist"
  docker run --name libra_cluster_test_builder_container -d -v $DIR/../..:/libra libra_cluster_test_builder sleep infinity
fi

docker exec libra_cluster_test_builder_container bash -c 'docker/cluster-test/compile.sh && /bin/cp /target/release/cluster-test target/cluster_test_docker_builder/'

if [ "$1" = "--build-binary" ]; then
  echo "Not building docker container, run without --build-binary to build docker image"
  echo "Built binary is in target/cluster_test_docker_builder/cluster-test"
else
  cp target/cluster_test_docker_builder/cluster-test cluster_test_docker_builder_cluster_test
  docker build -f $DIR/cluster-test.container.Dockerfile $DIR/../.. --tag cluster-test:latest $PROXY
  rm cluster_test_docker_builder_cluster_test
  echo
  echo
  echo "Built docker container, run with --build-binary to build binary only"
  echo "To run cluster test use:"
  echo "   docker run -t --rm cluster-test:latest <params>"
  echo "Mount /libra_rsa if you need to use SSH in cluster test:"
  echo "   docker run -v /libra_rsa:/libra_rsa -t --rm cluster-test:latest <params>"
fi
