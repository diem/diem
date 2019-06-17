#!/bin/sh

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -e

# Allowed Features
allowed_features=(
  "--and" "--not" "-e" "futures_api"
  "--and" "--not" "-e" "async_await"
  "--and" "--not" "-e" "await_macro"
  "--and" "--not" "-e" "box_patterns"
  "--and" "--not" "-e" "panic_info_message"
  "--and" "--not" "-e" "try_trait"
  "--and" "--not" "-e" "test"
  "--and" "--not" "-e" "specialization"
  "--and" "--not" "-e" "wait_timeout_until"
  "--and" "--not" "-e" "vec_remove_item"
)

# Search for nightly features
if git grep -e"#\!\[feature(.*)\]" "${allowed_features[@]}" -- "*.rs" >/dev/null 2>&1; then
  echo "Disallowed Nightly Features Found:"
  git grep -e"#\!\[feature(.*)\]" "${allowed_features[@]}" -- "*.rs"
  exit 1
else
  exit 0
fi
