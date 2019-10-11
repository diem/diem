#!/bin/sh
set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
if [ "$https_proxy" ]; then
    PROXY=" --build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
fi

rm -f libra-node-from-docker

docker build -f $DIR/validator_builder.Dockerfile $DIR/../.. --tag libra-node-builder $PROXY
TARGET=$DIR/../../target/libra-node-builder/
mkdir -p $TARGET
if docker ps -f name=libra-node-builder-container -f ancestor=libra-node-builder | grep libra-node-builder-container ; then
  echo "Builder container is running with latest builder image"
else
  echo "Builder image has changed, re-creating builder container"
  docker rm -f libra-node-builder-container || echo "Build container did not exist"
  docker run --name libra-node-builder-container -d -i -t -v $DIR/../..:/libra libra-node-builder bash
fi

docker exec -i -t libra-node-builder-container bash -c 'source /root/.cargo/env; cargo build -p libra-node --target-dir /target && /bin/cp /target/debug/libra-node target/libra-node-builder/'

ln target/libra-node-builder/libra-node libra-node-from-docker
docker build -f $DIR/validator.Dockerfile $DIR/../.. --tag libra_e2e --build-arg GIT_REV="$(git rev-parse HEAD)" --build-arg GIT_UPSTREAM="$(git merge-base HEAD origin/master)" --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" $PROXY
rm -f libra-node-from-docker