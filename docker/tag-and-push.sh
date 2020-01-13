#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
# tag-and-push.sh is used tag an image with multiple tags and push them to the target repo. Use ":" as the separator
# between multiple tags
# Example:
# SOURCE=libra_validator:latest TARGET_REPO=1234567890.dkr.ecr.us-west-2.amazonaws.com/libra_cluster_test TARGET_TAGS=master:master_39cnja0 tag-and-push.sh

set -e

TARGET_TAGS_ARR=(${TARGET_TAGS//:/ })
for TAG in "${TARGET_TAGS_ARR[@]}"
do
  TARGET=${TARGET_REPO}:${TAG}
  echo "Tagging ${SOURCE} to ${TARGET}"
  docker tag ${SOURCE} ${TARGET}
  echo "Pushing ${SOURCE} to ${TARGET}"
  docker push ${TARGET}
done
