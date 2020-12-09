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
  echo "Copies a diem dockerhub image to aws ecr"
  echo "dockerhub_to_ecr.sh -t <dockerhub_tag> [ -r <REPO> ]"
  echo "-t the tag that exists in hub.docker.com."
  echo "-o override tag that should be pushed to target repo."
  echo "-r target repo of image push"
  echo "-h this message"
}

DOCKERHUB_TAG=;
OUTPUT_TAG=;
TARGET_REPO=docker.io

#parse args
while getopts "t:o:r:h" arg; do
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
docker pull --disable-content-trust=false docker.io/libra/init:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/libra/faucet:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/libra/tools:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/libra/validator:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/libra/validator_tcb:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/libra/cluster_test:"$DOCKERHUB_TAG"
docker pull --disable-content-trust=false docker.io/libra/client:"$DOCKERHUB_TAG"

export DOCKER_CONTENT_TRUST=0

#Push the proper locations to novi ecr.
docker tag libra/init:"$DOCKERHUB_TAG" "$TARGET_REPO"/diem/init:"$OUTPUT_TAG"
docker tag libra/faucet:"$DOCKERHUB_TAG" "$TARGET_REPO"/diem/faucet:"$OUTPUT_TAG"
docker tag libra/tools:"$DOCKERHUB_TAG" "$TARGET_REPO"/diem/tools:"$OUTPUT_TAG"
docker tag libra/validator:"$DOCKERHUB_TAG" "$TARGET_REPO"/diem/validator:"$OUTPUT_TAG"
docker tag libra/validator_tcb:"$DOCKERHUB_TAG" "$TARGET_REPO"/diem/validator_tcb:"$OUTPUT_TAG"
docker tag libra/cluster_test:"$DOCKERHUB_TAG" "$TARGET_REPO"/diem/cluster_test:"$OUTPUT_TAG"
docker tag libra/client:"$DOCKERHUB_TAG" "$TARGET_REPO"/diem/client:"$OUTPUT_TAG"

docker push --disable-content-trust=true "$TARGET_REPO"/diem/init:"$OUTPUT_TAG"
docker push --disable-content-trust=true "$TARGET_REPO"/diem/faucet:"$OUTPUT_TAG"
docker push --disable-content-trust=true "$TARGET_REPO"/diem/tools:"$OUTPUT_TAG"
docker push --disable-content-trust=true "$TARGET_REPO"/diem/validator:"$OUTPUT_TAG"
docker push --disable-content-trust=true "$TARGET_REPO"/diem/validator_tcb:"$OUTPUT_TAG"
docker push --disable-content-trust=true "$TARGET_REPO"/diem/cluster_test:"$OUTPUT_TAG"
docker push --disable-content-trust=true "$TARGET_REPO"/diem/client:"$OUTPUT_TAG"

set +x
