# export SLACK_URL="<fill for slack integration>"
export ALLOW_DEPLOY=yes
unset AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_SESSION_TOKEN

mv run.out run.out.bak || echo 'run.out does not exist'
if ! ./cluster-test --workplace cluster-test --run > run.out ; then
  REQUEST=\{\"text\":\"cluster_test\ terminated\"\}
  curl -X POST -H 'Content-type: application/json' --data "$REQUEST" "$SLACK_URL"
  exit 1
fi
