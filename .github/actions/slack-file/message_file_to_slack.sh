#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

function echoerr() {
  cat <<< "$@" 1>&2;
}

# Check prerequists.
function check_command() {
    for var in "$@"; do
      if ! (command -v "$var" >/dev/null 2>&1); then
          echoerr "This command requires $var to be installed"
          exit 1
      fi
    done
}
check_command jq curl tr echo getopts

function usage() {
  echoerr -f Payload file.
  echoerr -w Webhook to which the message will be published.
  echoerr -d print debug messages.
  echoerr -h this message.
  echoerr variables it could calculate.
}

#Optional: if a private repo
PAYLOAD_FILE=;
WEBHOOK=;
DEBUG=false;
URL=;

while getopts 'f:w:u:d' OPTION; do
  case "$OPTION" in
    f)
      PAYLOAD_FILE="$OPTARG"
      ;;
    w)
      WEBHOOK="$OPTARG"
      ;;
    u)
      URL="$OPTARG"
      ;;
    d)
      DEBUG=true
      ;;
    ?)
      usage
      exit 1
      ;;
  esac
done

if [[ "${DEBUG}" == "true" ]]; then
  set -x
fi

if [ -e "${PAYLOAD_FILE}" ]; then
    jq -n \
    --arg msg "$(cat "${PAYLOAD_FILE}")" \
    --arg url "${URL}" \
    '{
        attachments: [
        {
            text: $msg,
            actions: [
            {
                "type": "button",
                "text": "Visit Job",
                "url": $url
            }
            ],
        }
        ]
    }' > /tmp/payload1234567890
    cat /tmp/payload1234567890
    if [ "${WEBHOOK}" ]; then
        curl -X POST -H 'Content-type: application/json' -d @/tmp/payload1234567890 "${WEBHOOK}"
    else
        echo "Not sending messages as no webhook url is set."
        echo "Chances are you are not building on master, or gha is misconfigured."
        echo "webhook is empty"
        exit 0
    fi
fi
