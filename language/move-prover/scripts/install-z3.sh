#!/bin/bash -e

# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

curl -LO https://github.com/Z3Prover/z3/releases/download/z3-4.8.9/z3-4.8.9-x64-osx-10.14.6.zip
unzip z3-4.8.9-x64-osx-10.14.6.zip
sudo cp z3-4.8.9-x64-osx-10.14.6/bin/z3 /usr/local/bin/
rm -rf z3-4.8.9-x64-osx-10.14.6
rm -rf z3-4.8.9-x64-osx-10.14.6.zip
