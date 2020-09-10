#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

######################################################################################################################
# Takes a slug org/repo ( libra/client ) and deletes all tags with release-* over 90 days and all other              #
# over 2 days (assumed to be test images).                                                                           #
######################################################################################################################

user=
pass=
dryrun=true
shouldfail=false

usage() {
  echo -u dockerhub username
  echo -p dockerhub password
  echo -x do not perform a dry run, delete images.
  echo -h this message.
  echo deletes release-* tags over 90 days old, and other over 2 days old.
  echo Done in shell, there is some TZ/leap second slop.
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

######################################################################################################################
# Takes a slug org/repo ( libra/client ) and deletes all tags with release-* over 90 days and all other
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
    RELEASES=`curl -L -s "https://registry.hub.docker.com/v2/repositories/${REPO}/tags?page_size=100" | \
    jq '."results"[] | (.name + " " + (.last_updated | sub(".[0-9]+Z$"; "Z") | fromdate | tostring ))' | \
    sed 's/"//g'`

    # Examples:
    #test-1_45e924ac 1597875188
    #test-2_fcbd9b94 1597871057
    #release-0.19_9424b8bd 1597857514
    #release-0.19_79e9afb2 1597296064
    #release-0.19_ba12e695 1597280940
    #testflow1_1057f73b 1597116042
    #testflow1_99db5fd1 1596143127

    NOW=`date "+%s"`
    #yeah leapseconds, dont care
    NOW_DAYS=`expr $NOW / 86400`

    echo NOW $NOW
    echo NOW_DAYS $NOW_DAYS

    echo "$RELEASES" | while IFS= read -r line ; do
        TAG=`echo $line | cut -d' ' -f1`
        TIME=`echo $line | cut -d' ' -f2`
        DAYS_SINCE_0=`expr $TIME / 86400`;
        AGE_DAYS=`expr $NOW_DAYS - $DAYS_SINCE_0`;

        DELETE=false
        if [[ $TAG == "release-"* ]] && [[ $AGE_DAYS -gt 90 ]]; then
            echo $REPO:$TAG is a release. It\'s age is $AGE_DAYS -- deleting
            DELETE=true
        elif [[ $TAG != "release-"* ]] && [[ $AGE_DAYS -gt 14 ]]; then
            echo $REPO:$TAG not release. It\'s age is $AGE_DAYS -- deleting
            DELETE=true
        else
            echo $REPO:$TAG is new, leaving alone.
        fi
        if [[ $DELETE == "true" ]] && [[ $dryrun == "false" ]]; then
            curl -X DELETE -u "$user:$pass" https://cloud.docker.com/v2/repositories/$REPO/tags/$TAG/
        fi
    done
}

prune_repo "libra/client"
prune_repo "libra/cluster_test"
prune_repo "libra/init"
prune_repo "libra/mint"
prune_repo "libra/tools"
prune_repo "libra/validator"
prune_repo "libra/validator_tcb"
#We currently overwrite, no need to delete.
#prune_repo "libra/build_environment"
