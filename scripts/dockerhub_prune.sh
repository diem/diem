#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

######################################################################################################################
# Takes a slug org/repo ( diem/client ) and deletes all tags with release-* over 90 days and all other              #
# over 2 days (assumed to be test images).                                                                           #
######################################################################################################################

user=
pass=
dryrun=true

usage() {
  echo "-u dockerhub username"
  echo "-p dockerhub password"
  echo "-x do not perform a dry run, delete images."
  echo "-h this message."
  echo "deletes release-* tags over 180 days old, and other over 7 days old."
  echo "Done in shell, there is some TZ/leap second slop."
}

while getopts 'u:p:xh' OPTION; do
  case "$OPTION" in
    u)
      user="$OPTARG"
      ;;
    p)
      pass="$OPTARG"
      ;;
    x)
      dryrun="false"
      ;;
    ?)
      usage
      exit 1
      ;;
  esac
done


login_data() {
  USERNAME=$1
  PASSWORD=$2
cat <<EOF
{
  "username": "$USERNAME",
  "password": "$PASSWORD"
}
EOF
}

#logs in and fetches a token
function get_token {
  USERNAME="$1"
  PASSWORD="$2"
  LOGIN_DATA=$(login_data "$USERNAME" "$PASSWORD")
  TOKEN=$(curl -s -H "Content-Type: application/json" -X POST -d "$LOGIN_DATA" "https://hub.docker.com/v2/users/login/" | jq -r .token)
  echo "$TOKEN"
}

#Deletes an individual tag from a repo slug, requires a token from get_token()
function del_tag {
  REPO="$1"
  TAG="$2"
  TOKEN="$3"
  curl "https://hub.docker.com/v2/repositories/${REPO}/tags/${TAG}/" \
  -X DELETE \
  -H "Authorization: JWT ${TOKEN}"
  OUTPUT=$?
  if [[ $OUTPUT -eq 0 ]]; then
    echo Deleted "$REPO:$TAG"
  else
    echo Failed to delete "$REPO:$TAG"
  fi
}


######################################################################################################################
# Takes a slug org/repo ( diem/client ) and deletes all tags with release-* over 180 days and all other
# over 2 days (assumed to be test images).
######################################################################################################################
function prune_repo {
    REPO=$1
    # Whoookay...
    # Follow closely on this one liner.
    # curl dockerhub to get a wad of json including the name/last updated date of each tag in the dockerhub repo.
    # The tag comes back as the .name field in the results.
    # The date formate isn't quite parsable by jq. See https://github.com/stedolan/jq/issues/1117.
    # So we hack it.  The string loses daylight savings time info as we convert in to seconds since epoch for testing.
    #
    # We don't care since there will be slop on when this script runs in CI, since it will run on the first commit in a day.
    #
    # Finally we sed of double quotes, and shove the values in to RELEASES for processing.
    RELEASES=$(curl -L -s "https://registry.hub.docker.com/v2/repositories/${REPO}/tags?page_size=100" | \
    jq '."results"[] | (.name + " " + (.last_updated | sub(".[0-9]+Z$"; "Z") | fromdate | tostring ))' | \
    sed 's/"//g')

    # Examples:
    #test-1_45e924ac 1597875188
    #test-2_fcbd9b94 1597871057
    #release-0.19_9424b8bd 1597857514
    #release-0.19_79e9afb2 1597296064
    #release-0.19_ba12e695 1597280940
    #testflow1_1057f73b 1597116042
    #testflow1_99db5fd1 1596143127

    NOW=$(date "+%s")
    #yeah leapseconds, dont care
    NOW_DAYS=$(( NOW / 86400 ))

    TO_DELETE=
    PAGE=0
    while [[ $(echo "$RELEASES" | wc -l) -gt 1 ]]; do
      PAGE=$(( PAGE + 1))
      while IFS= read -r line; do
          TAG=$(echo "$line" | cut -d' ' -f1)
          TIME=$(echo "$line" | cut -d' ' -f2)
          DAYS_SINCE_0=$(( TIME / 86400));
          AGE_DAYS=$(( NOW_DAYS - DAYS_SINCE_0 ));

          if [[ $TAG == "release-"* ]] && [[ $AGE_DAYS -gt 180 ]]; then
              echo "$REPO:$TAG is a release. It's age is $AGE_DAYS -- will delete"
              TO_DELETE="${TO_DELETE}"'
              '"${TAG}"
          elif [[ $TAG != "release-"* ]] && [[ $AGE_DAYS -gt 7 ]]; then
              echo "$REPO:$TAG not release. It's age is $AGE_DAYS -- will delete"
              TO_DELETE="${TO_DELETE}"'
              '"${TAG}"
          else
              echo "$REPO:$TAG is new, leaving alone."
          fi
      done <<< "$RELEASES"
      echo PAGE="$PAGE"
      RELEASES=$(curl -L -s "https://registry.hub.docker.com/v2/repositories/${REPO}/tags?page_size=100&page=${PAGE}" | \
      jq '."results"[] | (.name + " " + (.last_updated | sub(".[0-9]+Z$"; "Z") | fromdate | tostring ))' | \
      sed 's/"//g')
  done

  if [[ $dryrun == "false" ]]; then
    TOKEN=$( get_token "$user" "$pass" )
    while IFS= read -r TAG; do
      TAG=${TAG// /}
      if [[ "$TAG" != "" ]]; then
        del_tag "$REPO" "$TAG" "$TOKEN"
      fi
    done <<< "$TO_DELETE"
  else
    echo Dry run, not deleting tags:
    echo "$TO_DELETE"
  fi

}

prune_repo "diem/client"
prune_repo "diem/cluster_test"
prune_repo "diem/init"
prune_repo "diem/faucet"
prune_repo "diem/tools"
prune_repo "diem/validator"
prune_repo "diem/validator_tcb"
prune_repo "diem/forge"

#We currently overwrite, no need to delete.
#prune_repo "diem/build_environment"
