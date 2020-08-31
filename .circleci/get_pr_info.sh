#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

#Check prerequists.
function check_command() {
    for var in "$@"; do
      if ! (command -v "$var" >/dev/null 2>&1); then
          echo "This command requires $var to be installed"
          exit 1
      fi
    done
}
check_command jq curl tr echo getopts

function usage() {
  echo -u github username \(only needed for private repos\).
  echo -p github password \(only needed for private repos\).
  echo -g previous githash to compare against, in many cases this can be calculated.
  echo -b if we are using bors, bors-ng, etc, use pr commit messages to determine target branch for merge if in \'auto\' branch.
  echo -h this message.
  echo output will be echoed environment variables
  echo GITHUB_SLUG, the repo that has been written to, or a pr is attempting to write to.
  echo TARGET_BRANCH, the git branch that a pr is attempting to write to, if this build is for a pr.
  echo BASE_GITHASH, if the build is pr, the merge-base, if a push, the last build of this branch using the -g parameter.
  echo CHANGED_FILE_OUTPUTFILE the path to a temp file which is the delta between the BASE_GITHASH and the current build\'s HEAD
  echo if the changed file set cannot be calculated for any reason the script will return -1, after echoing out all the
  echo variables it could calculate.
}

USERSET_PREVIOUS_GITHASH=;
#Optional: if a private repo
GITHUB_USERNAME=;
GITHUB_PASSWORD=;
BORS=false

while getopts 'u:p:g:bh' OPTION; do
  case "$OPTION" in
    u)
      GITHUB_USERNAME="$OPTARG"
      ;;
    p)
      GITHUB_PASSWORD="$OPTARG"
      ;;
    g)
      USERSET_PREVIOUS_GITHASH="$OPTARG"
      ;;
    b)
      BORS="true"
      ;;
    ?)
      usage
      exit 1
      ;;
  esac
done

#calculate the login for curl
LOGIN="";
if [[ -n "${GITHUB_USERNAME}" ]] && [[ -n "${GITHUB_PASSWORD}" ]]; then
  LOGIN="--user ${GITHUB_USERNAME}:${GITHUB_PASSWORD}"
fi

# Find the git repo slug.   If gha use GITHUB_REPOSITORY, if CircleCi use CIRCLE_PROJECT_USERNAME/CIRCLE_PROJECT_REPONAME
# else use the git remote origin, and sed to guess.  Computes from https/git urls.
if [[ -n "${GITHUB_REPOSITORY}" ]]; then
  GITHUB_SLUG=${GITHUB_REPOSITORY}
elif  [[ -n  "${CIRCLE_PROJECT_REPONAME}" ]] && [[ -n  "${CIRCLE_PROJECT_USERNAME}" ]]; then
  GITHUB_SLUG="${CIRCLE_PROJECT_USERNAME}/${CIRCLE_PROJECT_REPONAME}"
else
  GITHUB_SLUG="$(git remote get-url --all origin | sed 's/.git$//g' | sed 's/http[s]:\/\/[^\/]*\///' | sed 's/.*://')"
fi

BRANCH=
#Attempt to determine/normalize the branch.
if [[ "${CIRCLECI}" == "true" ]]; then
  if [[ -n "$CIRCLE_BRANCH" ]]; then
    BRANCH=${CIRCLE_BRANCH}
  fi
elif  [[ -n "$GITHUB_ACTION" ]]; then
  if [[ -n "$GITHUB_REF" ]]; then
    BRANCH=${GITHUB_REF}
  fi
fi

#If we have a git hash, and it exist in the history of the current HEAD, then use it as BASE_GITHASH
if [[ -n "$USERSET_PREVIOUS_GITHASH" ]] && git merge-base --is-ancestor "$USERSET_PREVIOUS_GITHASH" "$(git rev-parse HEAD)" 2>/dev/null ; then
  BASE_GITHASH=${USERSET_PREVIOUS_GITHASH}
fi

#If we can find the PR number it overrides any user input git hash.
#if we are on the "auto" branch use the commit message to determine bor's target branch.
# Bor's target branch -- use bors commit message as source of truth.
if [[ "${BORS}" == "true" ]] && [[ "${BRANCH}" == "auto" ]]; then
  commit_message=$(git log -1 --pretty=%B)
  PR_NUMBER=$(echo "${commit_message}" | tail -1 | sed 's/Closes: #//')
else
  #Let's see if this is a pr.
  #If circle
  if [[ -n "$CIRCLE_PR_NUMBER" ]]; then
    PR_NUMBER="$CIRCLE_PR_NUMBER"
  elif [[ -n "$CIRCLE_BRANCH" ]] && [[ "$CIRCLE_BRANCH" =~ pull/.* ]]; then
    PR_NUMBER=$(echo "$CIRCLE_BRANCH" | sed 's/pull\///')
  #Multiple circle pull requests can use the same githash... if their's only one, use it.
  elif [[ -n "$CIRCLE_PULL_REQUESTS" ]] && [[ ! "$CIRCLE_PULL_REQUESTS" =~ "," ]]; then
    PR_NUMBER=$(echo "$CIRCLE_PULL_REQUESTS" | sed 's/.*\///')
  #If github actions
  elif [[ -n "$GITHUB_REF" ]] && [[ "$GITHUB_REF" =~ "/pull/" ]]; then
    PR_NUMBER=$(echo "$GITHUB_REF" | sed 's/pull\///' | sed 's/\/merge//')
  fi
fi

#look up the pr with the pr number
if [[ -n ${PR_NUMBER} ]]; then
  PR_RESPONSE=$(curl -s "$LOGIN" "https://api.github.com/repos/${GITHUB_SLUG}/pulls/$PR_NUMBER")
  if [[ -n ${PR_RESPONSE} ]] && [[ $(echo "$PR_RESPONSE" | jq -e '.message') != '"Not Found"' ]]; then
    #PR_TITLE=$(echo "$PR_RESPONSE" | jq -e '.title' | tr -d '"')
    #PR_AUTHOR=$(echo "$PR_RESPONSE" | jq -e '.user.login' | tr -d '"')
    PR_BASE_BRANCH=$(echo "$PR_RESPONSE" | jq -e '.base.ref' | tr -d '"')
  fi

  #if we have a branch name, and the exact branch exists.
  if [[ -n "$PR_BASE_BRANCH" ]] && [[ $(git branch -l --no-color | tr -d '*' | tr -d ' ' | grep -c '^'"$PR_BASE_BRANCH"'$') == 1 ]]; then
    BASE_GITHASH=$(git merge-base HEAD origin/"$PR_BASE_BRANCH")
    TARGET_BRANCH="$PR_BASE_BRANCH"
  fi
fi

if [[ -n "$BASE_GITHASH" ]]; then
  CHANGED_FILE_OUTPUTFILE=$(mktemp /tmp/changed_files.XXXXXX)
  git --no-pager diff --name-only "$BASE_GITHASH" | sort > "$CHANGED_FILE_OUTPUTFILE"
fi

echo 'export PULL_REQUEST='"$PR_NUMBER"
echo 'export BASE_GITHASH='"$BASE_GITHASH"
echo 'export CHANGED_FILE_OUTPUTFILE='"$CHANGED_FILE_OUTPUTFILE"
echo 'export TARGET_BRANCH='"$TARGET_BRANCH"
