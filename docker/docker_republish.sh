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
TARGET_REPO="docker.io"
TARGET_ORG="diem"
DISABLE_TRUST="false"
IMAGES="init faucet tools validator validator_tcb cluster_test client forge"

#parse args
while getopts "t:o:r:g:i:dh" arg; do
  case $arg in
    t)
      DOCKERHUB_TAG="$OPTARG"
      ;;
    o)
      OUTPUT_TAG="$OPTARG"
      ;;
    r)
      TARGET_REPO="$OPTARG"
      ;;
    g)
      TARGET_ORG="$OPTARG"
      ;;
    d)
      DISABLE_TRUST=true
      ;;
    i)
      IMAGES="$OPTARG"
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

IFS=' ' read -ra TARGET_IMAGES <<< "$IMAGES"

for image in "${TARGET_IMAGES[@]}"; do
  renamed_image="${image//-/_}"
  docker pull --disable-content-trust=false docker.io/diem/"$renamed_image":"$DOCKERHUB_TAG"
done

if [[ $DISABLE_TRUST == "true" ]]; then
  export DOCKER_CONTENT_TRUST=0
fi

# retag images for new location
for image in "${TARGET_IMAGES[@]}"; do
  renamed_image="${image//-/_}"
  docker tag diem/"$renamed_image":"$DOCKERHUB_TAG" "$TARGET_REPO"/"$TARGET_ORG"/"$renamed_image":"$OUTPUT_TAG"
done

#Push the proper locations to novi ecr.



tmpfile=$(mktemp)
success=0;
echo "Failed to republish" >> "${tmpfile}"
for image in "${TARGET_IMAGES[@]}"; do
  renamed_image="${image//-/_}"
  docker push --disable-content-trust="$DISABLE_TRUST" "$TARGET_REPO"/"$TARGET_ORG"/"$renamed_image":"$OUTPUT_TAG" || success=$(echo "$renamed_image" >> "${tmpfile}"; echo 1)
done

if [[ "$success" == "1" ]]; then
  cat "${tmpfile}"
fi

set +x
exit "${success}"
