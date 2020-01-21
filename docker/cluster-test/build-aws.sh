#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
# build-aws.sh builds the cluster-test image on AWS Codebuild and pushes the image to AWS ECR.
# Examples:
# build-aws.sh --version pull/123 --addl_tags test_tag # Builds the image from pull request 123 and tags it with dev_pull_123,dev_u29a020,test_tag
# build-aws.sh --version u29a020  --addl_tags test_tag # Builds the image from commit u29a020 and tags it with dev_u29a020,test_tag

$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )/../build-aws-base.sh --project libra-cluster-test "$@"
