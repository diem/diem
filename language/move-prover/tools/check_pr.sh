#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# A script to check whether a local commit is ready for a PR.

set -e

BASE=$(git rev-parse --show-toplevel)
echo "*************** [check-pr] Assuming diem root at $BASE"

CRATES="\
  $BASE/language/move-prover/bytecode \
  $BASE/language/move-prover/boogie-backend \
  $BASE/language/move-prover\
  $BASE/language/move-model\
"

while getopts "hxd" opt; do
  case $opt in
    h)
      echo "Runs cargo fmt and cargo xclippy on prover relevant crates."
      echo "By default, only core crates are checked. With option -x,"
      echo "more crates are included. With option -d, also Move library"
      echo "crates are run. With -g, diem's git-check script is run".
      echo "Use -xdg to enable everything."
      exit 1
      ;;
    x)
      CRATES="$CRATES \
        $BASE/language/move-prover/abigen\
        $BASE/language/move-prover/docgen\
        $BASE/language/move-prover/errmapgen\
        $BASE/language/move-prover/interpreter\
        $BASE/language/move-prover/interpreter-testsuite\
        $BASE/language/move-prover/lab\
        $BASE/language/move-prover/test-utils\
        "
      ;;
    d)
      RUN_CRATES="
        $BASE/language/move-stdlib\
        $BASE/language/diem-framework\
      "
      ;;
    g)
      GIT_CHECKS=1
      ;;
  esac
done


for dir in $CRATES; do
  echo "*************** [check-pr] Checking crate $dir"
  (
    cd $dir;
    cargo xfmt
    cargo xclippy
  )
done

for dir in $RUN_CRATES; do
   echo "*************** [check-pr] Running crate $dir"
   (
     cd $dir;
     cargo run
   )
done

if [ ! -z "$GIT_CHECKS" ]; then
   echo "*************** [check-pr] Diem git checks"
   $BASE/scripts/git-checks.sh
fi
