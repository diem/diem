#!/bin/sh

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -e

# Allowed Features
allowed_features=(
  "--and" "--not" "-e" "async_await"
  "--and" "--not" "-e" "checked_duration_since"
  "--and" "--not" "-e" "crate_visibility_modifier"
  "--and" "--not" "-e" "drain_filter"
  "--and" "--not" "-e" "exhaustive_patterns"
  "--and" "--not" "-e" "never_type"
  "--and" "--not" "-e" "panic_info_message"
  "--and" "--not" "-e" "repeat_generic_slice"
  "--and" "--not" "-e" "set_stdio"
  "--and" "--not" "-e" "slice_concat_ext"
  "--and" "--not" "-e" "specialization"
  "--and" "--not" "-e" "test"
  "--and" "--not" "-e" "trait_alias"
)

# Search for nightly features
if git grep -e"#\!\[feature(.*)\]" "${allowed_features[@]}" -- "*.rs" >/dev/null 2>&1; then
  echo "Disallowed Nightly Features Found:"
  git grep -e"#\!\[feature(.*)\]" "${allowed_features[@]}" -- "*.rs"
  exit 1
else
  exit 0
fi
