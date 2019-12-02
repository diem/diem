#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

declare -a params
if [ -n "${IPV4}" ]; then
    params+="-4 "
fi
if [ -n "${BOOTSTRAP}" ]; then
    params+="-b ${BOOTSTRAP} "
fi
if [ -n "${NODE}" ]; then
    params+="-i ${NODE} "
fi
if [ -n "${LISTEN}" ]; then
    params+="-l ${LISTEN} "
fi
if [ -n "${NODES}" ]; then
    params+="-n ${NODES} "
fi
if [ -n "${SEED}" ]; then
    params+="-s ${SEED} "
fi
if [ -n "${TEMPLATE}" ]; then
    params+="-t ${TEMPLATE} "
fi

/opt/libra/bin/test-node ${params[@]}
