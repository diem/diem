#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# GCC that comes with XCode generates extra whitespace lines too keep line numbers in file intact
# This is inconsistent with gcc v9 that we have in CI that removes empty lines
# It means need to use gcc v9 locally(can be installed with brew), otherwise CI will complaint that files are different
if which gcc-9 > /dev/null; then
    GCC=gcc-9
else
    if (gcc -v 2>&1 | grep 'Apple clang'); then
        echo "Looks like you are building on Mac :)"
        echo "You don't seem to have gcc-9 and your gcc is from XCode"
        echo "You need to install gcc from brew:"
        echo "   brew install gcc"
        exit 1
    else
        GCC=gcc
    fi
fi

echo "Using $GCC"

$GCC -x c -P -E -o docker/cluster-test/cluster-test.Dockerfile docker/cluster-test/cluster-test.Dockerfile.template
$GCC -DINCREMENTAL_IMAGE -x c -P -E -o docker/cluster-test/cluster-test.container.Dockerfile docker/cluster-test/cluster-test.Dockerfile.template
