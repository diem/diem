#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Expects these environment variables
if [ -z $VERSION ] || [ -z $ADDL_TAG ]; then
    echo "Must specify image VERSION and ADDL_TAG to build"
    exit 1
fi

set +e
date
RETRYABLE_EXIT_CODE=2
for ((i = 0; i < 3; i++)); do
    echo "Build attempt $i"
    docker/build-aws.sh --build-all-cti --version $VERSION --addl_tags canary,${ADDL_TAG}
    return_code=$?
    if [[ $return_code -eq 0 ]]; then
        echo "Build successful"
        exit 0
    fi
    if [[ $return_code -ne ${RETRYABLE_EXIT_CODE} ]]; then
        echo "Build failed"
        exit 1
    fi
    echo "Retrying build"
done
echo "Build failed after retries"
exit 1
