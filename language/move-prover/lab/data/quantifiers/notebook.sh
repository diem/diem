#!/bin/sh
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0


export BASE="$(git rev-parse --show-toplevel)/language/move-prover/lab/data/quantifiers"

jupyter lab ${BASE}/notebook.ipynb
