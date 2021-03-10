#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -e

oldrev="origin/main"
newrev="HEAD"

# Disallow merge commits
for rev in $(git rev-list --merges $oldrev..$newrev); do
	echo "Merge commit $rev found"
	echo "Please fix your branch/PR to not include merges"
	exit 1
done

# Disallow submodules
for rev in $(git rev-list $oldrev..$newrev); do
	comparison=$(git rev-parse --quiet --verify $rev^ || true)

	# Submodules are indicated by files with mode 160000
	submodules=$(git diff-tree -r $comparison $rev | awk '$2 ~ /160000/ {print substr($0, index($0,$6))}' || true)
	if [[ $submodules ]] ; then
		echo "Git submodules are not allowed."
		echo "Commit $rev introduces a submodule at path: $submodules"
		exit 1
	fi
done

# Disallow checking in multiple files with the same name on case-insensitive
# filesystems on OSes like macOS or Windows. These files work fine in linux,
# but break git checkouts on case-insensitive filesystems.
export LC_ALL=C
for rev in $(git rev-list $oldrev..$newrev); do
	test -z "$(git diff-tree $rev -r --root --no-commit-id --name-only \
		--diff-filter=ACR)" && continue

	names=$(git ls-tree -r --name-only $rev | \
		awk '{fn=tolower($0); files[fn]++; if (files[fn] > 1) {print fn}}')

	if [[ -n $names ]]; then
		echo "Commit $rev would introduce files with the same name on"
		echo "case-insensitive filesystems (common on macOS and Windows)."
		echo "Bad filenames: $names"
		exit 1
	fi
done

# Disallow checking in insane filenames that are not broadly
# compatible across source control systems and plaforms.
#
# For example, no carriage returns or line feeds should be allowed in filenames.
export LC_ALL=C
for rev in $(git rev-list $oldrev..$newrev); do
	names=$(git diff-tree $rev -r --root --no-commit-id --name-only \
		--diff-filter=AR)

	# Block commits with filenames outside normal ASCII
	badnames=$(grep '\\\|[`{}|~:]' <<< "$names") || true
	#                ^^ find anything with a backslash in the name
	#                  ^^ or...
	#                    ^^^^^^^ any of these funky chars

if [[ -n $badnames ]]; then
	echo "Bad filenames in commits are blocked to protect the codebase"
	echo "from side-effects on different platforms."
	echo "Bad filenames: $badnames"
	exit 1
fi
done

# If there are whitespace errors, print the offending file names and fail.
for rev in $(git rev-list $oldrev..$newrev); do
	comparison=$(git rev-parse --quiet --verify $rev^ || true)

	whitespace=$(git diff --check $comparison $rev || true)
	if [[ $whitespace ]] ; then
		echo "Found whitespace errors in commit $rev:"
		echo
		git diff --check $comparison $rev
		exit 1
	fi
done
