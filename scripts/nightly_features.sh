#!/bin/sh

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -e

# Allowed Features
allowed_features=(
  "--and" "--not" "-e" "async_await"
  "--and" "--not" "-e" "checked_duration_since"
  "--and" "--not" "-e" "custom_test_frameworks"
  "--and" "--not" "-e" "set_stdio"
  "--and" "--not" "-e" "test"
)

# Search for nightly features
if git grep -e"#\!\[feature(.*)\]" "${allowed_features[@]}" -- "*.rs" >/dev/null 2>&1; then
  echo "Disallowed Nightly Features Found:"
  git grep -e"#\!\[feature(.*)\]" "${allowed_features[@]}" -- "*.rs"
  exit 1
else
  exit 0
fi
