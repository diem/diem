#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

crate=
retries=
shouldfail=false

usage() {
  echo -r number of times to retry
  echo -f fail of the script does not succeed at least once
  echo -c crates to run
}

while getopts 'fr:c:' OPTION; do
  case "$OPTION" in
    f)
      shouldfail=true
      ;;
    r)
      retries=$OPTARG
      ;;
    c)
      crate="$OPTARG"
      ;;
    ?)
      usage
      exit 1
      ;;
  esac
done

if ! [[ "$retries" =~ ^[0-9]+$ ]]; then usage; exit 1; fi

if [ -z "$crate" ]; then usage; exti 1; fi

if [[ "$shouldfail" == false ]]; then echo command will never fail; fi

count=0
success=-1
RUST_BACKTRACE=1
while [ $count -lt $retries ] && [ $success -ne 0 ]
do
  cargo xtest --package $crate && success=0
  count=$(( $count + 1 ))
done

if [ "$success" -eq "0" ] || [ "$shouldfail" == false ]; then
   echo Crates: $crate succeeded after $count executions.
   exit 0;
else
   echo Crates: $crate failed after $count executions.
   exit 1;
fi
