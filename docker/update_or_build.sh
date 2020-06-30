#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

function usage {
  echo "Usage:"
  echo "update_or_build.sh [-p] -g <GITHASH> -b <TARGET_BRANCH> -n <image name>"
  echo "-p indicates this a prebuild, where images are built and pushed to dockerhub with an infix of '_pre_', should be run on the 'auto' branch, trigger by bors."
  echo "-g the GIT_HASH of the form: short=8"
  echo "-b the branch we're building on, or the branch we're targeting if a prebuild"
  echo "-n name, one of init, mint, validator, validator-dynamic, safety-rules, cluster-test"
  echo "should be called from the root folder of the libra project, and must have it's .git history"
}


PREBUILD=false;
NAME=
BRANCH=

#parse args
while getopts "pb:n:" arg; do
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

#If not a prebuild *attempt* to pull the previously built image.
if [ $PREBUILD != "true" ]; then
  docker pull libra/test:libra_${NAME}_pre_${BRANCH}_${GIT_REV}
  export PULLED=$?
fi

#Convert dashes to underscores to get tag names, except for validator, which for some reason is "e2e"
tag_name=`echo $NAME | sed 's/-/_/g'`
if [ $NAME == "validator" ]; then
  tag_name=e2e;
fi

#if a prebuild, always -1, else if "docker pull" failed build the image.
if [ "$PULLED" != "0" ]; then
  docker/$NAME/build.sh
  if [ $NAME == "cluster-test" ]; then
    echo retagging cluster-test as libra/test:libra_${tag_name}_pre_${BRANCH}_${GIT_REV}
    docker tag cluster-test libra/test:libra_${tag_name}_pre_${BRANCH}_${GIT_REV}
  else
    echo retagging libra_${tag_name} as libra/test:libra_${tag_name}_pre_${BRANCH}_${GIT_REV}
    docker tag libra_${tag_name} libra/test:libra_${tag_name}_pre_${BRANCH}_${GIT_REV}
  fi
  #push our tagged prebuild image if this is a prebuild.  Usually means this is called from bors' auto branch.
  if [ $PREBUILD == "true" ]; then
     echo pushing libra/test:libra_${tag_name}_pre_${BRANCH}_${GIT_REV}
     docker push libra/test:libra_${tag_name}_pre_${BRANCH}_${GIT_REV}
  fi
fi

#if not a prebuild tag and push, usually means this is called from a release branch
if [ $PREBUILD != "true" ]; then
  echo retagging libra/test:libra_${tag_name}_pre_${BRANCH}_${GIT_REV} as libra/test:libra_${tag_name}_${BRANCH}_${GIT_REV}
  docker tag libra/test:libra_${tag_name}_pre_${BRANCH}_${GIT_REV} libra/test:libra_${tag_name}_${BRANCH}_${GIT_REV}
  echo pushing libra/test:libra_${tag_name}_${BRANCH}_${GIT_REV}
  docker push libra/test:libra_${tag_name}_${BRANCH}_${GIT_REV}
fi
