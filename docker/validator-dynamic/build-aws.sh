#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
# build-aws.sh builds the validator-dynamic image on AWS Codebuild and pushes the image to AWS ECR.
# Examples:
# build-aws.sh --version pull/123 --addl_tags test_tag # Builds the image from pull request 123 and tags it with dev_pull_123,dev_u29a020,test_tag
# build-aws.sh --version u29a020  --addl_tags test_tag # Builds the image from commit u29a020 and tags it with dev_u29a020,test_tag

if ! which jq &>/dev/null; then
  echo "jq is not installed. Please install jq"
  exit 1
fi

function print_usage() {
  echo "Usage:"
  echo "build-aws.sh --version pull/123 --addl_tags test_tag"
  echo "build-aws.sh --version u29a020 --addl_tags test_tag"
  exit 1
}

while [[ "$1" =~ ^- ]]; do case $1 in
  --version )
    shift;
    if [[ "$1" =~ ^pull ]]; then
      SOURCE_VERSION="refs/${1}/head"
      TAGS="dev_${1/\//_}"
    else
      SOURCE_VERSION="${1}"
      TAGS="dev_${1}"
    fi
    ;;
  --addl_tags )
    shift;
    ADDL_TAGS="${1}"
    ;;
  --help )
    shift;
    print_usage
    ;;
esac; shift; done

if [ -z "${SOURCE_VERSION}" ]; then
    print_usage
fi

if [[ -n "${ADDL_TAGS}" ]]; then
  TAGS="${TAGS},${ADDL_TAGS}"
fi

echo "Building with SOURCE_VERSION=${SOURCE_VERSION} TAGS=${TAGS}"

# Use : as the separator because environment-variables-override does not allow
# comma in its specification
BUILD_ID=$(aws codebuild start-build --project-name libra-validator \
 --environment-variables-override name=TAGS,value=${TAGS//,/:} \
 --source-version ${SOURCE_VERSION} | jq -r .build.id)

if [ -z "${BUILD_ID}" ]; then
  echo "Failed to submit build"
  exit 1
fi

echo "Started build with ID ${BUILD_ID}"

while true; do
    BUILD_JSON=$(aws codebuild batch-get-builds --ids ${BUILD_ID})
    BUILD_STATUS=$(echo $BUILD_JSON | jq -r '.builds[0].buildStatus')
    CURRENT_PHASE=$(echo $BUILD_JSON | jq -r '.builds[0].currentPhase')
    if [ $BUILD_STATUS = "IN_PROGRESS" ]; then
        echo "Build in progress. Current phase: ${CURRENT_PHASE}..."
        sleep 10
    elif [ $BUILD_STATUS = "SUCCEEDED" ]; then
        echo "Build and push completed successfully"
        exit
    else
        echo "Build failed with status : ${BUILD_STATUS}"
        exit 1
    fi
done
