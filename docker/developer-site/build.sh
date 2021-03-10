#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"

docker build -f "$DIR/Dockerfile" "$DIR/../../developers.diem.com"  --tag diem/developer-site:latest
