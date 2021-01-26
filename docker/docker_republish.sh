#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

#############################################################################################
# Takes previously published dockerhub images and publishes them to your logged in registry.
# The assumption is local tagged images are available in dockerhub, and you are not logged in
# to dockerhub, but likely AWS ECR, or similar.
#############################################################################################

function usage {
  echo "Usage:"
  echo "Copies a diem dockerhub image to another docker location (server / repo / tag )"
  echo "docker_republish.sh -t <dockerhub_tag> [ -r <REPO> ]"
  echo "-t the tag that exists in hub.docker.com."
  echo "-o override tag that should be pushed to target repo."
  echo "-r target repo of image push"
  echo "-g target org, defaults to 'diem'"
  echo "-d disable signing for target repository (ecr requires this)"
  echo "-h this message"
}

DOCKERHUB_TAG=;
OUTPUT_TAG=;
TARGET_REPO=docker.io
TARGET_ORG=diem
DISABLE_TRUST=false

#parse args
while getopts "t:o:r:g:dh" arg; do
  case $arg in
    t)
      DOCKERHUB_TAG=$OPTARG
      ;;
    o)
      OUTPUT_TAG=$OPTARG
      ;;
    r)
      TARGET_REPO=$OPTARG
      ;;
    g)
      TARGET_ORG=$OPTARG
      ;;
    d)
      DISABLE_TRUST=true
      ;;
    *)
      usage;
      exit 0;
      ;;
  esac
done

[[ "$DOCKERHUB_TAG" == "" ]] && { echo DOCKERHUB_TAG not set; usage; exit; }
if [[  "$OUTPUT_TAG" == "" ]]; then
  echo OUTPUT_TAG not set, using "$DOCKERHUB_TAG"
  OUTPUT_TAG=$DOCKERHUB_TAG
fi

set -x

#Pull the latest docker hub images so we can push them to ECR
docker pull --disable-content-trust=false docker.io/diem/init:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/diem/faucet:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/diem/tools:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/diem/validator:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/diem/validator_tcb:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/diem/cluster_test:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/diem/client:"$DOCKERHUB_TAG"

if [[ $DISABLE_TRUST == "true" ]]; then
  export DOCKER_CONTENT_TRUST=0
fi

#Push the proper locations to novi ecr.
docker tag diem/init:"$DOCKERHUB_TAG" "$TARGET_REPO"/"$TARGET_ORG"/init:"$OUTPUT_TAG"
docker tag diem/faucet:"$DOCKERHUB_TAG" "$TARGET_REPO"/"$TARGET_ORG"/faucet:"$OUTPUT_TAG"
docker tag diem/tools:"$DOCKERHUB_TAG" "$TARGET_REPO"/"$TARGET_ORG"/tools:"$OUTPUT_TAG"
docker tag diem/validator:"$DOCKERHUB_TAG" "$TARGET_REPO"/"$TARGET_ORG"/validator:"$OUTPUT_TAG"
docker tag diem/validator_tcb:"$DOCKERHUB_TAG" "$TARGET_REPO"/"$TARGET_ORG"/validator_tcb:"$OUTPUT_TAG"
docker tag diem/cluster_test:"$DOCKERHUB_TAG" "$TARGET_REPO"/"$TARGET_ORG"/cluster_test:"$OUTPUT_TAG"
docker tag diem/client:"$DOCKERHUB_TAG" "$TARGET_REPO"/"$TARGET_ORG"/client:"$OUTPUT_TAG"

docker push --disable-content-trust="$DISABLE_TRUST" "$TARGET_REPO"/"$TARGET_ORG"/init:"$OUTPUT_TAG"
docker push --disable-content-trust="$DISABLE_TRUST" "$TARGET_REPO"/"$TARGET_ORG"/faucet:"$OUTPUT_TAG"
docker push --disable-content-trust="$DISABLE_TRUST" "$TARGET_REPO"/"$TARGET_ORG"/tools:"$OUTPUT_TAG"
docker push --disable-content-trust="$DISABLE_TRUST" "$TARGET_REPO"/"$TARGET_ORG"/validator:"$OUTPUT_TAG"
docker push --disable-content-trust="$DISABLE_TRUST" "$TARGET_REPO"/"$TARGET_ORG"/validator_tcb:"$OUTPUT_TAG"
docker push --disable-content-trust="$DISABLE_TRUST" "$TARGET_REPO"/"$TARGET_ORG"/cluster_test:"$OUTPUT_TAG"
docker push --disable-content-trust="$DISABLE_TRUST" "$TARGET_REPO"/"$TARGET_ORG"/client:"$OUTPUT_TAG"

set +x
