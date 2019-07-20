#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -e

for rev in $(git rev-list origin/master..HEAD); do
  if [ "$(git cat-file -p "$rev" | grep --count parent)" -ne 1 ]; then
    echo "Merge commit $rev found"
    echo "Please fix your branch/PR to not include merges"
    exit 1
  fi
done
