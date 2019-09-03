#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Check for modified or untracked files after CI has run
changes="$(git status --porcelain)"
echo "${changes}"
[[ -z "${changes}" ]]
