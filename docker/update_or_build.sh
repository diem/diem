#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

function usage {
  echo "Usage:"
  echo "build and properly tags images for deployment to docker hub"
  echo "update_or_build.sh [-p] -g <GITHASH> -b <TARGET_BRANCH> -n <image name> [-u]"
  echo "-p indicates this a prebuild, where images are built and pushed to dockerhub with an infix of '_pre_', should be run on the 'auto' branch, trigger by bors."
  echo "-b the branch we're building on, or the branch we're targeting if a prebuild"
  echo "-n name, one of init, mint, validator, validator-dynamic, safety-rules, cluster-test"
  echo "-u 'upload', or 'push' the docker images will be pushed to dockerhub, otherwise only locally tag"
  echo "should be called from the root folder of the libra project, and must have it's .git history"
}

PREBUILD=false;
NAME=
BRANCH=
PUSH=false

#parse args
while getopts "pb:n:u" arg; do
  case $arg in
    p)
      PREBUILD="true"
      ;;
    n)
      NAME=$OPTARG
      ;;
    b)
      BRANCH=$OPTARG
      ;;
    u)
      PUSH="true"
      ;;
    h)
      usage;
      exit 0;
      ;;
  esac
done

GIT_REV=$(git rev-parse --short=8 HEAD)

[ "$BRANCH" != "" ] || { echo "-b branch must be set"; usage; exit 99; }
[ "$GIT_REV" != "" ] || { echo "Could not determine git revision, aborting"; usage; exit 99; }
[ "$NAME" != "" ] || { echo "-n name must be set"; usage; exit 99; }

PULLED="-1"

#Convert dashes to underscores to get tag names, except for validator, which for some reason is "e2e"
tag_name=`echo $NAME | sed 's/-/_/g'`
if [ $NAME == "validator" ]; then
  tag_name="e2e";
fi

#The name of the docker image built in the "auto" branch
PRE_NAME=libra/test:libra_${tag_name}_pre_${BRANCH}_${GIT_REV}

#If not a prebuild *attempt* to pull the previously built image.
if [ $PREBUILD != "true" ]; then
  docker pull ${PRE_NAME}
  export PULLED=$?
fi

#if a prebuild, always -1, else if "docker pull" failed build the image.
if [ "$PULLED" != "0" ]; then
  docker/$NAME/build.sh
  #tag the built image with consistent naming...  ugh cluster-test, and e2e
  if [ $NAME == "cluster-test" ]; then
    echo retagging cluster-test as ${PRE_NAME}
    docker tag cluster-test ${PRE_NAME}
  else
    echo retagging libra_${tag_name} as ${PRE_NAME}
    docker tag libra_${tag_name} ${PRE_NAME}
  fi
  #push our tagged prebuild image if this is a prebuild.  Usually means this is called from bors' auto branch.
  if [ $PREBUILD == "true" ]; then
    if [ $PUSH == "true" ]; then
      echo pushing ${PRE_NAME}
      docker push ${PRE_NAME}
    else
      echo Dry run, Not pushing ${PRE_NAME}
    fi
  fi
fi

#if not a prebuild tag and push, usually means this is called from a release branch
if [ $PREBUILD != "true" ]; then
  PUB_NAME=libra/test:libra_${tag_name}_${BRANCH}_${GIT_REV}
  echo retagging ${PRE_NAME} as ${PUB_NAME}
  docker tag ${PRE_NAME} ${PUB_NAME}
  if [ $PUSH == "true" ]; then
    echo pushing ${PUB_NAME}
    docker push ${PUB_NAME}
  else
    echo Dry run, Not pushing ${PUB_NAME}
  fi
fi
