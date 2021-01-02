#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

function echoerr() {
  cat <<< "$@" 1>&2;
}

#Check prerequists.
function check_command() {
    for var in "$@"; do
      if ! (command -v "$var" >/dev/null 2>&1); then
          echoerr "This command requires $var to be installed"
          exit 1
      fi
    done
}
check_command jq curl tr echo getopts git

function usage() {
  echoerr -u github username \(only needed for private repos\).
  echoerr -p github password \(only needed for private repos\).
  echoerr -w workflow file name, used to find old runs in pushed updates for git base revision comparison.
  echoerr -b if we are using bors, bors-ng, etc, use pr commit messages to determine target branch for merge if in \'auto\' branch.
  echoerr -d print debug messages.
  echoerr -h this message.
  echoerr output will be echoed environment variables
  echoerr GITHUB_SLUG, the repo that has been written to, or a pr is attempting to write to.
  echoerr TARGET_BRANCH, the git branch that a pr is attempting to write to, if this build is for a pr.
  echoerr BASE_GITHASH, if the build is pr, the merge-base, if a push, the last build of this branch using the -g parameter.
  echoerr CHANGED_FILE_OUTPUTFILE the path to a temp file which is the delta between the BASE_GITHASH and the current build\'s HEAD
  echoerr if the changed file set cannot be calculated for any reason the script will return -1, after echoing out all the
  echoerr variables it could calculate.
}

#Optional: if a private repo
GITHUB_USERNAME=;
GITHUB_PASSWORD=;
BORS=false
DEBUG=false
WORKFLOW_FILE=;

while getopts 'u:p:g:bdw:h' OPTION; do
  case "$OPTION" in
    u)
      GITHUB_USERNAME="$OPTARG"
      ;;
    p)
      GITHUB_PASSWORD="$OPTARG"
      ;;
    b)
      BORS=true
      ;;
    d)
      DEBUG=true
      ;;
    w)
      WORKFLOW_FILE="$OPTARG"
      ;;
    ?)
      usage
      exit 1
      ;;
  esac
done

if [[ -z "${WORKFLOW_FILE}" ]]; then
  echoerr Workflow file must be sepecified.
  usage
  exit 1
fi

#calculate the login for curl
LOGIN="";
if [[ -n "${GITHUB_USERNAME}" ]] && [[ -n "${GITHUB_PASSWORD}" ]]; then
  LOGIN="--user ${GITHUB_USERNAME}:${GITHUB_PASSWORD}"
fi

# Find the git repo slug.   If gha use GITHUB_REPOSITORY, if CircleCi use CIRCLE_PROJECT_USERNAME/CIRCLE_PROJECT_REPONAME
# else use the git remote origin, and sed to guess.  Computes from https/git urls.
if [[ -n "${GITHUB_REPOSITORY}" ]]; then
  GITHUB_SLUG=${GITHUB_REPOSITORY}
else
  GITHUB_SLUG="$(git remote get-url --all origin | sed 's/.git$//g' | sed 's/http[s]:\/\/[^\/]*\///' | sed 's/.*://')"
fi
$DEBUG && echo GITHUB_SLUG="$GITHUB_SLUG"

BRANCH=
#Attempt to determine/normalize the branch.
if  [[ "$GITHUB_EVENT_NAME" == "push" ]]; then
  BRANCH=${GITHUB_REF//*\/}
fi
if  [[ "$GITHUB_EVENT_NAME" == "pull_request" ]]; then
  BRANCH=${GITHUB_BASE_REF}
  TARGET_BRANCH=${BRANCH}
fi
$DEBUG && echoerr BRANCH="$BRANCH"

BASE_GITHASH=
if [[ "${GITHUB_EVENT_NAME}" == "push" ]] && [[ "$BORS" == false || ( "$BRANCH" != "auto" && "$BRANCH" != "canary" ) ]]; then
  $DEBUG && echoerr URL="${GITHUB_API_URL}/repos/${GITHUB_REPOSITORY}/actions/workflows/${WORKFLOW_FILE}/runs?branch=${BRANCH}&status=completed&event=push"
  QUERIED_GITHASH="$( curl "$LOGIN" -H 'Accept: application/vnd.github.v3+json' "${GITHUB_API_URL}/repos/${GITHUB_REPOSITORY}/actions/workflows/${WORKFLOW_FILE}/runs?branch=${BRANCH}&status=completed&event=push" 2>/dev/null | jq '.workflow_runs[0].head_sha' | sed 's/"//g')"
  $DEBUG && echoerr QUERIED_GITHASH="$QUERIED_GITHASH"
  #If we have a git hash, and it exist in the history of the current HEAD, then use it as BASE_GITHASH
  if [[ -n "$QUERIED_GITHASH" ]] && [[ $(git merge-base --is-ancestor "$QUERIED_GITHASH" "$(git rev-parse HEAD)" 2>/dev/null; echo $?) == 0 ]]; then
    BASE_GITHASH="${QUERIED_GITHASH}"
  fi
fi
$DEBUG && echoerr BASE_GITHASH="$BASE_GITHASH"

#If we can find the PR number it overrides any user input git hash.
#if we are on the "auto"/"canary" branch use the commit message to determine bor's target branch.
#Bor's target branch -- use bors commit message as source of truth.
if [[ "$BORS" == true ]] &&  [[ "$BRANCH" == "auto" || "$BRANCH" == "canary" ]] ; then
  commit_message=$(git log -1 --pretty=%B)
  PR_NUMBER=$(echo "${commit_message}" | tail -1 | sed 's/Closes: #//')
else
  #Let's see if this is a pr.
  #If github actions
  if [[ -n "$GITHUB_REF" ]] && [[ "$GITHUB_REF" =~ "/pull/" ]]; then
    PR_NUMBER=$(echo "$GITHUB_REF" | sed 's/refs\/pull\///' | sed 's/\/merge//')
  fi
fi
$DEBUG && echoerr PR_NUMBER="$PR_NUMBER"

#look up the pr with the pr number
if [[ -n ${PR_NUMBER} ]] && [[ -z ${TARGET_BRANCH} ]]; then
  PR_RESPONSE=$(curl -s "$LOGIN" "${GITHUB_API_URL}/repos/${GITHUB_SLUG}/pulls/$PR_NUMBER")
  if [[ -n ${PR_RESPONSE} ]] && [[ $(echo "$PR_RESPONSE" | jq -e '.message') != '"Not Found"' ]]; then
    PR_BASE_BRANCH=$(echo "$PR_RESPONSE" | jq -e '.base.ref' | tr -d '"')
  fi
  #if we have a branch name, and the exact branch exists.
  if [[ -n "$PR_BASE_BRANCH" ]] && [[ $(git branch -l --no-color | tr -d '*' | tr -d ' ' | grep -c '^'"$PR_BASE_BRANCH"'$') == 1 ]]; then
    TARGET_BRANCH="$PR_BASE_BRANCH"
  fi
fi
$DEBUG && echoerr TARGET_BRANCH="$TARGET_BRANCH"
if [[ -n "$TARGET_BRANCH" ]] && [[ -z "$BASE_GITHASH" ]]; then
  BASE_GITHASH=$(git merge-base HEAD origin/"$TARGET_BRANCH")
fi

if [[ -n "$BASE_GITHASH" ]]; then
  CHANGED_FILE_OUTPUTFILE=$(mktemp /tmp/changed_files.XXXXXX)
  git --no-pager diff --name-only "$BASE_GITHASH" | sort > "$CHANGED_FILE_OUTPUTFILE"
fi

echo 'export CHANGES_PULL_REQUEST='"$PR_NUMBER"
echo 'export CHANGES_BASE_GITHASH='"$BASE_GITHASH"
echo 'export CHANGES_CHANGED_FILE_OUTPUTFILE='"$CHANGED_FILE_OUTPUTFILE"
echo 'export CHANGES_TARGET_BRANCH='"$TARGET_BRANCH"
