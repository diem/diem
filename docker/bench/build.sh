#!/bin/sh
set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
if [ -z ${https_proxy+x} ]; then
  PROXY="--build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
fi

echo "Building base..."
BASE_BUILDER_DIR=$DIR/../base
cd $BASE_BUILDER_DIR && pwd && ./build.sh

echo "Building client..."
docker build \
  -f $DIR/bench.Dockerfile \
  $DIR/../.. \
  --tag libra_bench \
  --build-arg GIT_REV="$(git rev-parse HEAD)" \
  --build-arg GIT_UPSTREAM="$(git merge-base HEAD origin/master)" \
  --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
  $PROXY
