#! /bin/bash

TRACE_PATH=$HOME/trace

[ ! -e  "$TRACE_PATH" ] || rm -f "$TRACE_PATH"

export MOVE_VM_TRACE=$TRACE_PATH

cat foo > "$1"
cat foo > "$2"

unset MOVE_VM_TRACE

echo "DONE"
