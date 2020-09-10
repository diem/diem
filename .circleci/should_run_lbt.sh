#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -eo pipefail

oldrev="origin/master"
newrev="HEAD"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR=$SCRIPT_DIR/..
IGNORE_PATHS_FILE="$SCRIPT_DIR/ignore_paths.json"

# Look up ignore paths
IGNORE_PATHS=$(cat $IGNORE_PATHS_FILE | jq -r '.ignore_paths|join(" ")')

match_ignore_paths() {
  _file=$1
  _ignore=0
  for i in $IGNORE_PATHS; do
    if [[ "$_file" == "$i"* ]]; then
      _ignore=1
      break
    fi
  done
  echo $_ignore
}

# Look up updated files in the PR
cd "$PROJECT_DIR"
FILES=$(git diff --name-only $oldrev..$newrev)

should_run=false
for f in $FILES; do
  if [[ $(match_ignore_paths $f) == 0 ]]; then
    should_run=true
    echo "Found at least one file not in ignore-list: $f"
    break
  fi
done

if $should_run ; then
  echo "Will run land-blocking test."
  exit 0
else
  echo "Will skip land-blocking test."
  exit 1
fi
