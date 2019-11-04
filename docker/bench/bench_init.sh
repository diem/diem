#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

echo "$MINT_KEY" | base64 -d > /opt/libra/etc/mint.key

/opt/libra/bin/ruben -a $AC_HOST -f /opt/libra/etc/mint.key --metrics-server-address "0.0.0.0:9101" $@
