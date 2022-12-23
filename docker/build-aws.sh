#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0
# build-aws-base.sh is a common script shared by mutiple build-aws.sh scripts

NON_RETRY_EXIT_CODE=1
RETRYABLE_EXIT_CODE=2

if ! which jq &>/dev/null; then
  echo "jq is not installed. Please install jq"
  exit 1
fi

function print_usage() {
  echo "Usage:"
  echo "$(basename $0) --build-all --version pull/123 --addl_tags test_tag"
  echo "$(basename $0) --build-validator --version u29a020 --addl_tags test_tag"
  exit 1
}

USER=$(whoami)
BUILD_PROJECTS=()

while [[ "$1" =~ ^- ]]; do case $1 in
  --build-all )
    BUILD_PROJECTS=(diem-validator diem-cluster-test diem-init diem-faucet diem-safety-rules diem-tools)
    ;;
    # NOTE: This is used in land-blocking job `.github/workflows/ci-test.yml`
    #       If you change the list of projects to be built for `--build-all-cti`, please
    #       change the list in `.github/actions/land-blocking/find-lbt-images.sh` as well
  --build-all-cti )
    BUILD_PROJECTS=(diem-validator diem-cluster-test diem-init diem-safety-rules)
    ;;
  --build-cluster-test )
    BUILD_PROJECTS=(diem-cluster-test)
    ;;
  --build-validator )
    BUILD_PROJECTS=(diem-validator)
    ;;
  --build-init )
    BUILD_PROJECTS=(diem-init)
    ;;
  --build-faucet )
      BUILD_PROJECTS=(diem-faucet)
    ;;
  --build-safety-rules )
    BUILD_PROJECTS=(diem-safety-rules)
    ;;
  --build-tools )
    BUILD_PROJECTS=(diem-tools)
    ;;
  --build-forge )
    BUILD_PROJECTS=(diem-forge diem-validator diem-init diem-faucet diem-safety-rules diem-tools)
    ;;
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
  --help )
    shift;
    print_usage
    ;;
esac; shift; done

if [ -z "${SOURCE_VERSION}" ] || [ ${#BUILD_PROJECTS[@]} -eq 0 ]; then
    print_usage
fi

if [[ -n "${ADDL_TAGS}" ]]; then
  TAGS="${TAGS},${ADDL_TAGS}"
fi

echo "Building with SOURCE_VERSION=${SOURCE_VERSION} TAGS=${TAGS}"

submit_build() {
    local PROJECT="${1}"
    # Use : as the separator because environment-variables-override does not allow
    # comma in its specification
    BUILD_ID=$(aws codebuild start-build --project-name "${PROJECT}" \
    --environment-variables-override name=TAGS,value=${TAGS//,/:} \
      name=ENABLE_FAILPOINTS,value=$ENABLE_FAILPOINTS \
    --source-version ${SOURCE_VERSION} | jq -r .build.id)

    if [ -z "${BUILD_ID}" ]; then
        echo "Failed to submit build for ${PROJECT}. Make sure you have proper AWS credentials in your environment."
        exit 1
    fi

    echo "Started build for project ${PROJECT} with ID ${BUILD_ID}. Link to the build https://us-west-2.console.aws.amazon.com/codesuite/codebuild/projects/${PROJECT}/build/${BUILD_ID}/"
}

get_build_status() {
    BUILD_STATUS=""
    CURRENT_PHASE=""
    DOWNLOAD_SOURCE_STATUS=""
    local BUILD_ID="${1}"
    local BUILD_JSON=$(aws codebuild batch-get-builds --ids ${BUILD_ID})
    if [[ $? -gt 0 ]]; then
      echo "aws codebuild batch-get-builds failed"
      return $?
    fi
    BUILD_STATUS=$(echo $BUILD_JSON | jq -r '.builds[0].buildStatus')
    CURRENT_PHASE=$(echo $BUILD_JSON | jq -r '.builds[0].currentPhase')
    DOWNLOAD_SOURCE_STATUS=$(echo $BUILD_JSON | jq -r '.builds[0].phases[] | select(.phaseType == "DOWNLOAD_SOURCE") | .phaseStatus')
}

# not using bash associative arrays because OSX ships with old bash version which does not support this
BUILD_IDS=()
for (( i=0; i < ${#BUILD_PROJECTS[@]}; i++ ));
do
   submit_build ${BUILD_PROJECTS[$i]}
   BUILD_IDS+=("${BUILD_ID}")
done

while true; do
    ALL_SUCCEEDED=true
    for (( i=0; i < ${#BUILD_PROJECTS[@]}; i++ ));
    do
        get_build_status ${BUILD_IDS[$i]}
        if [[ ${DOWNLOAD_SOURCE_STATUS} == "FAILED" ]]; then
          echo "${BUILD_PROJECTS[$i]} download source failed"
          exit ${NON_RETRY_EXIT_CODE}
        fi
        if [[ $BUILD_STATUS == "TIMED_OUT" ]]; then
          echo "${BUILD_PROJECTS[$i]} build timed out"
          exit ${RETRYABLE_EXIT_CODE}
        fi
        if [[ $? -gt 0 ]] || [[ ${BUILD_STATUS} == "" ]]; then
          ALL_SUCCEEDED=false
        elif [[ $BUILD_STATUS == "FAILED" ]] || [[ $BUILD_STATUS == "FAULT" ]] || [[ $BUILD_STATUS == "STOPPED" ]]; then
          echo "${BUILD_PROJECTS[$i]} build failed with status ${BUILD_STATUS}"
          exit ${NON_RETRY_EXIT_CODE}
        elif [[ ${BUILD_STATUS} != "SUCCEEDED" ]]; then
          ALL_SUCCEEDED=false
        fi
        echo "${BUILD_PROJECTS[$i]} current phase: ${CURRENT_PHASE}"
    done
    if [[ $ALL_SUCCEEDED == true ]]; then
        echo "All builds completed successfully"
        exit 0
    fi
    sleep 30
done
