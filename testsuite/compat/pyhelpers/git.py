#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
from .cli import execute_cmd_with_text_output

# length of git hash prefix
SHORT_LEN = 7
TAG_PREFIX = "master_"
LBT_SHORT_LEN = 8
LBT_TAG_PREFIX = "land_"


def _fetch(remote: str = "origin") -> None:
    execute_cmd_with_text_output(
        ["git", "fetch", remote],
        err="Something wrong with git. Are you in the right dir?",
    )


def latest_testnet_tag() -> str:
    """Get the image tag based on the latest revision in branch"""
    _fetch()
    short_rev = execute_cmd_with_text_output(
        ["git", "log", "origin/testnet", '--format=format:"%h"', "-1"]
    ).strip('"')[:SHORT_LEN]
    return TAG_PREFIX + short_rev


def prev_tag(n: int) -> str:
    """Get the previous nth commit's image tag"""
    _fetch()
    short_rev = execute_cmd_with_text_output(
        ["git", "rev-parse", f"--short={LBT_SHORT_LEN}", f"HEAD~{n}"]
    )
    return LBT_TAG_PREFIX + short_rev
