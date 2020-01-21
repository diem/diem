#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
# build-aws-base.sh is a common script shared by mutiple build-aws.sh scripts

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
      TAGS="dev_${USER}_${1/\//_}"
    else
      SOURCE_VERSION="${1}"
      TAGS="dev_${USER}_${1}"
    fi
    ;;
  --addl_tags )
    shift;
    ADDL_TAGS="${1}"
    ;;
  --project )
    shift;
    PROJECT="${1}"
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
BUILD_ID=$(aws codebuild start-build --project-name "${PROJECT}" \
 --environment-variables-override name=TAGS,value=${TAGS//,/:} \
 --source-version ${SOURCE_VERSION} | jq -r .build.id)

if [ -z "${BUILD_ID}" ]; then
  echo "Failed to submit build. Make sure you have proper AWS credentials in your environment."
  exit 1
fi

echo "Started build with ID ${BUILD_ID}. Link to the build https://us-west-2.console.aws.amazon.com/codesuite/codebuild/projects/${PROJECT}/build/${BUILD_ID}/"

while true; do
    BUILD_JSON=$(aws codebuild batch-get-builds --ids ${BUILD_ID})
    BUILD_STATUS=$(echo $BUILD_JSON | jq -r '.builds[0].buildStatus')
    CURRENT_PHASE=$(echo $BUILD_JSON | jq -r '.builds[0].currentPhase')
    if [ $BUILD_STATUS = "IN_PROGRESS" ]; then
        echo "Build in progress. Current phase: ${CURRENT_PHASE}..."
        sleep 30
    elif [ $BUILD_STATUS = "SUCCEEDED" ]; then
        echo "Build and push completed successfully"
        exit
    else
        echo "Build failed with status : ${BUILD_STATUS}"
        exit 1
    fi
done
