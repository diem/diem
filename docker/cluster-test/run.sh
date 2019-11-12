#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# export SLACK_LOG_URL="<fill for slack integration>"
# export SLACK_CHANGELOG_URL="<fill for slack integration>"
export ALLOW_DEPLOY=yes
unset AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_SESSION_TOKEN


while true; do
  mv run.out run.out.bak || echo 'run.out does not exist'
  REQUEST="{\"text\":\"cluster_test starting.\"}"
  curl -X POST -H 'Content-type: application/json' --data "$REQUEST" "$SLACK_URL"
  ./cluster-test --workplace cluster-test --run > run.out
  REQUEST="{\"text\":\"cluster_test terminated with status: $?.\"}"
  curl -X POST -H 'Content-type: application/json' --data "$REQUEST" "$SLACK_URL"
  sleep 30
done
