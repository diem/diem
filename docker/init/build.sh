#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
if [ "$https_proxy" ]; then
    PROXY=" --build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
fi

for ((i = 0; i < 30; i++)); do
  docker build -f $DIR/Dockerfile $DIR/../.. --tag libra_init --build-arg GIT_REV="$(git rev-parse HEAD)" --build-arg GIT_UPSTREAM="$(git merge-base HEAD origin/master)" --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" $PROXY "$@"
  if [[ $? -eq 0 ]]; then
    echo "docker build succeeded"
    exit 0
  fi
  echo "docker build failed. retrying in 10 secs."
  sleep 10
done
exit 1
