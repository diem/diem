#!/bin/bash -e

# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

PKG=z3-4.8.9-x64-ubuntu-16.04

curl -LO https://github.com/Z3Prover/z3/releases/download/z3-4.8.9/${PKG}.zip
unzip ${PKG}.zip
sudo cp ${PKG}/bin/z3 /usr/local/bin/
rm -rf ${PKG}
rm -rf ${PKG}.zip
