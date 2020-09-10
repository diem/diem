#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"

cd $DIR

docker build . --tag 853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest
docker push 853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest
