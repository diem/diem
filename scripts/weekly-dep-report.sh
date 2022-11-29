#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# fast fail.
set -eo pipefail

echo " Dependency change report "
echo "--------------------------"

# expects two parameters: repository name and branch
repository=$1
starter_point=${2:-"HEAD"}
git checkout --quiet "${starter_point}"

last_commit=$(git rev-list HEAD -1)
first_commit_within_last_seven_days=$(git rev-list --reverse --since=7.days.ago HEAD | head -n 1)
first_commit_in_the_repo=$(git rev-list HEAD | tail -n 1)

format_change_in_dependency_output () {
    # if negative, do nothing; if positive, add + prefix; if zero, return "="
    if [[ $1 -gt 0 ]]; then
        echo "+$1"
    elif [[ $1 == 0 ]]; then
        echo "="
    else
        echo "$1"
    fi
}

get_total_deps () {
    cargo guppy resolve-cargo --resolver-version v2 --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind thirdparty --omit-edges-into diem-workspace-hack | wc -l | tr -d ' '
}

get_total_direct_deps() {
    cargo guppy resolve-cargo --resolver-version v2 --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind directthirdparty --omit-edges-into diem-workspace-hack | wc -l | tr -d ' '
}

get_total_dsr_deps () {
    cargo guppy resolve-cargo --resolver-version v2 --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind thirdparty --omit-edges-into diem-workspace-hack -p safety-rules | wc -l | tr -d ' '
}

get_total_direct_dsr_deps () {
    cargo guppy resolve-cargo --resolver-version v2 --target-platform x86_64-unknown-linux-gnu --host-platform x86_64-unknown-linux-gnu --build-kind target --kind directthirdparty --omit-edges-into diem-workspace-hack -p safety-rules | wc -l | tr -d ' '
}

get_total_dups () {
    cargo guppy dups --target x86_64-unknown-linux-gnu | wc -l | tr -d ' '
}


git checkout --quiet "${last_commit}"
date=$(git show --no-patch --format=%cd --date='format:%Y-%m-%d')
short_sha=$(git rev-parse --short HEAD)
total=$(get_total_deps)
direct=$(get_total_direct_deps)
lsr_total=$(get_total_dsr_deps)
lsr_direct=$(get_total_direct_dsr_deps)
dups=$(get_total_dups)

if [[ -z "${first_commit_within_last_seven_days}" ]]; then
    # case 1: no commit made in the last seven days
    echo "No new commit made this week."
elif [[ "${first_commit_within_last_seven_days}" == "${first_commit_in_the_repo}" ]]; then
    # case 2: the repository started this week.
    echo "The repository doesn't have any commit before current week."
else
    # case 3: there is at least one commit in the last seven days
    last_commit_from_last_week=$(git rev-list "${first_commit_within_last_seven_days}"^1 -1)
    git checkout --quiet "${last_commit_from_last_week}"
fi

prev_date=$(git show --no-patch --format=%cd --date='format:%Y-%m-%d')
prev_short_sha=$(git rev-parse --short HEAD)
prev_total=$(get_total_deps)
prev_direct=$(get_total_direct_deps)
prev_lsr_total=$(get_total_dsr_deps)
prev_lsr_direct=$(get_total_direct_dsr_deps)
prev_dups=$(get_total_dups)

change_total=$(format_change_in_dependency_output $((total-prev_total)))
change_direct=$(format_change_in_dependency_output $((direct-prev_direct)))
change_lsr_total=$(format_change_in_dependency_output $((lsr_total-prev_lsr_total)))
change_lsr_direct=$(format_change_in_dependency_output $((lsr_direct-prev_lsr_direct)))
change_dups=$(format_change_in_dependency_output $((dups-prev_dups)))

# resetting HEAD commit
if [[ "${starter_point}" == "HEAD" ]]; then
    git checkout --quiet "${last_commit}"
else
    git checkout --quiet "${starter_point}"
fi

echo "Dependency change in $repository:$starter_point at Commit $short_sha ($date) since Commit $prev_short_sha ($prev_date) :"
echo "- total third party deps: $total ($change_total)"
echo "- direct third party deps: $direct ($change_direct)"
echo "- third party deps in DSR: $lsr_total ($change_lsr_total)"
echo "- direct third party deps in DSR: $lsr_direct ($change_lsr_direct)"
echo "- total duplicate deps: $dups ($change_dups)"
