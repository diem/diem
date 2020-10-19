#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

for target in $(cargo x test --package smoke-test -- --list | grep "::test" | cut -d ':' -f 3) ; do
    RUST_BACKTRACE=full cargo x test -p smoke-test -- $target --test-threads 1
done
