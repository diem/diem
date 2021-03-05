#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -e

# Regenerate bytecode dumps for all examples
EXAMPLES="
 modifies.move\
 opaque.move\
 references.move\
 resource.move\
 simple.move\
 "

for example in ${EXAMPLES}; do
  cargo run -- -k --dump-bytecode ${example}
done
