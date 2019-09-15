#!/bin/sh

echo "$MINT_KEY" | base64 -d > /opt/libra/etc/mint.key

/opt/libra/bin/ruben -a $AC_HOST -f /opt/libra/etc/mint.key --metrics_server_address "0.0.0.0:9101" $@
