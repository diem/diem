#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# A script to upgrade from `spec fun name` style syntax and related changes to `spec name`. This can create
# errors, specifically the last replacement for `define ` might be critical, so to be used with care.

# NOTE: this script is not idempotent. Running it twice will result in invalid syntax.

for file in "$@"; do
  echo "migrating $file"
  sed -i 's/spec fun/spec/' $file
  sed -i 's/spec struct/spec/' $file
  sed -i 's/spec define/spec fun/' $file
  sed -i 's/define /fun /' $file
  sed -i 's/ \[global\]//' $file
done
