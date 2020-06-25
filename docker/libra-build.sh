#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

DOCKERFILE=$1
TAG=$2

if [ -z "$DOCKERFILE" ] || [ -z "$TAG" ]
then
  echo "Usage libra-build.sh <Docker_file> <Local_tag> <Other_args>"
fi

shift 2

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
if [ "$https_proxy" ]; then
    PROXY=" --build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
fi

for _ in seq 1 2; do
  if docker build -f $DOCKERFILE $DIR/.. --tag $TAG \
    --build-arg GIT_REV="$(git rev-parse HEAD)" \
    --build-arg GIT_UPSTREAM="$(git merge-base HEAD origin/master)" \
    --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
    $PROXY \
    "$@"; then
      exit 0
  fi
  echo "docker build failed. retrying in 10 secs."
  sleep 10
done
exit 1
