#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

import json
from subprocess import check_call, check_output, DEVNULL, CalledProcessError


def execute_cmd_with_text_output(cmd: list, wdir: str = None, err: str = None) -> str:
    try:
        return check_output(cmd, cwd=wdir, stderr=DEVNULL, encoding="UTF-8").strip()
    except CalledProcessError:
        print(f"ERROR: {err}")
        raise


def execute_cmd_with_json_output(cmd: list, wdir: str = None, err: str = None) -> dict:
    try:
        out = check_output(cmd, cwd=wdir, stderr=DEVNULL)
        return json.loads(out)
    except CalledProcessError:
        print(f"ERROR: {err}")
        raise
