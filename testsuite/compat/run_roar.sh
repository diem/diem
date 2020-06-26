#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Wrapper around ROAR python script, enforcing pipenv

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

pipenv run python3 $DIR/record_and_replay.py $@
