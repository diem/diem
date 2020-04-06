#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -eo pipefail

oldrev="origin/master"
newrev="HEAD"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR=$SCRIPT_DIR/..

# Look for updated Dockerfiles in the PR
cd "$PROJECT_DIR"
COMMITS=$(git rev-list $oldrev..$newrev)
DOCKER_FILES=$(
  for commit in $COMMITS; do
    git diff-tree --no-commit-id --name-only -r "$commit" -- "*Dockerfile";
  done
)

if [ "$DOCKER_FILES" ]; then
  echo "Will build docker containers."
else
  echo "Will skip building docker containers."
  circleci step halt
fi
