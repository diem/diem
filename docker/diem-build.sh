#!/bin/sh
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

DOCKERFILE=$1
if [ -z "$DIEM_BUILD_TAG" ]; then
  TAGS="--tag $2"
else
  TAGS="--tag $2 --tag $DIEM_BUILD_TAG"
fi

RESTORE='\001\033[0m\002'
BLUE='\001\033[01;34m\002'
DOCKERFILE_BUILD_RE='^RUN ./docker/build-common.sh$'

if [ -z "$DOCKERFILE" ] || [ -z "$TAGS" ]
then
  echo "Usage diem-build.sh <Docker_file> <Local_tag> <Other_args>"
fi

shift 2

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
if [ "$https_proxy" ]; then
    PROXY=" --network host --build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
fi

if [ "$1" = "--incremental" ]; then
  shift
  if ! grep -E "$DOCKERFILE_BUILD_RE" "$DOCKERFILE" > /dev/null; then
    echo "It looks like $DOCKERFILE does not support incremental compilation"
    exit 1
  fi
  echo "Using incremental compilation"
  if ! docker ps > /dev/null; then
    echo "Docker seem not to be working, try installing docker?"
    exit 1
  fi
  TOOLCHAIN=$(cat "$DIR/../rust-toolchain")
  CONTAINER=builder-rust-${TOOLCHAIN}
  OUT_TARGET="$DIR/../target-out-docker"
  if ! (docker ps | grep "$CONTAINER" > /dev/null); then
    if (docker ps -a | grep "$CONTAINER" > /dev/null); then
      echo "${BLUE}Build container exists, but is not running, starting it${RESTORE}"
      docker start "$CONTAINER" || (printf "Failed to start build container, try\n\tdocker rm -f %" "$CONTAINER"; exit 1)
    else
      echo "${BLUE}Build container does not exist, setting up new one${RESTORE}"
      IMAGE=circleci/rust:${TOOLCHAIN}-buster
      mkdir -p "$OUT_TARGET"
      docker run -u root -d -t -v "$OUT_TARGET:/out-target" -v "$DIR/..:/diem" -w /diem --name "$CONTAINER" "$IMAGE" sh > /dev/null
      docker exec -i -t "$CONTAINER" apt-get update
      docker exec -i -t "$CONTAINER" apt-get install -y cmake curl clang git
      echo "${BLUE}Container is set up, starting build${RESTORE}"
    fi
  else
    echo "${BLUE}Running build in existing container.${RESTORE}"
    echo "${BLUE}Tip:${RESTORE}: If you see unexpected failure, try docker rm -f $CONTAINER"
  fi
  CONTAINER_TARGET=/target
  docker exec -e "STRIP_DIR=$CONTAINER_TARGET" -e "ENABLE_FAILPOINTS=$ENABLE_FAILPOINTS" -i -t "$CONTAINER" "./docker/build-common.sh" --target-dir "$CONTAINER_TARGET"
  echo "${BLUE}Build is done, copying over artifacts${RESTORE}"
  docker exec -i -t "$CONTAINER" sh -c 'find /target/release -maxdepth 1 -executable -type f | xargs -I F cp F /out-target'
  TMP_DOCKERFILE="$DOCKERFILE.tmp"
  trap 'rm -f $TMP_DOCKERFILE' EXIT
  sed -e "s+$DOCKERFILE_BUILD_RE+RUN mkdir -p /diem/target/release/; cp target-out-docker/* /diem/target/release/+g" \
      "$DOCKERFILE" > "$TMP_DOCKERFILE"
  DOCKERFILE=$TMP_DOCKERFILE
  echo "${BLUE}Starting docker build process${RESTORE}"
fi

for _ in seq 1 2; do
  if docker build -f $DOCKERFILE $DIR/.. $TAGS \
    --build-arg GIT_REV="$(git rev-parse HEAD)" \
    --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
    --build-arg ENABLE_FAILPOINTS="$ENABLE_FAILPOINTS" \
    $PROXY \
    "$@"; then
      exit 0
  fi
  echo "docker build failed. retrying in 10 secs."
  sleep 10
done
exit 1
