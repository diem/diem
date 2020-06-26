#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
from .cli import execute_cmd_with_text_output

# length of git hash prefix
SHORT_LEN = 7
TAG_PREFIX = "master_"


def _fetch(remote: str = "origin") -> None:
    execute_cmd_with_text_output(
        ["git", "fetch", remote],
        err="Something wrong with git. Are you in the right dir?",
    )


def latest_branch_tag(branch: str) -> str:
    """Get the image tag based on the latest revision in branch"""
    _fetch()
    short_rev = execute_cmd_with_text_output(
        ["git", "log", f"origin/{branch}", '--format=format:"%h"', "-1"]
    ).strip('"')[:SHORT_LEN]
    return TAG_PREFIX + short_rev


def prev_tag(n: int) -> str:
    """Get the previous nth commit's image tag"""
    _fetch()
    short_rev = execute_cmd_with_text_output(
        ["git", "rev-parse", f"--short={SHORT_LEN}", f"HEAD~{n}"]
    )
    return TAG_PREFIX + short_rev
