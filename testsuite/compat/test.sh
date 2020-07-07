#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

pipenv run pytest tests

pipenv run black --check .
