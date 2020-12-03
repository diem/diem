#!/bin/bash -e

# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

curl -L0 https://cvc4.cs.stanford.edu/downloads/builds/macos/cvc4-25-09-20.zip -o cvc4-25-09-20.zip
unzip cvc4-25-09-20.zip
sudo cp 25-09-20/cvc4 /usr/local/bin/
rm -rf 25-09-20
rm -rf cvc4-25-09-20.zip
