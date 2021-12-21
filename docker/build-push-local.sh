#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e
REPO=853397791086.dkr.ecr.us-west-2.amazonaws.com

aws ecr get-login-password \
    --region us-west-2 \
    | docker login \
    --username AWS \
    --password-stdin "$REPO"

BUILD_PROJECTS=(validator cluster-test init safety-rules)

TAG=${TAG:-"dev_$(whoami)_$(git rev-parse --short HEAD)"}
echo "[$(date)] Using tag $TAG"

for (( i=0; i < ${#BUILD_PROJECTS[@]}; i++ ));
do
   PROJECT=${BUILD_PROJECTS[$i]}
   export DIEM_BUILD_TAG="$REPO/diem_${PROJECT/-/_}:$TAG"
   DOCKER_BUILDER="$PROJECT"
   echo "[$(date)] Building $PROJECT via $DOCKER_BUILDER"
   "./docker/${DOCKER_BUILDER}/build.sh" --incremental
   echo "[$(date)] Pushing $PROJECT"
   time docker push "$DIEM_BUILD_TAG"
done

echo "[$(date)] Build complete"
