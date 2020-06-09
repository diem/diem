#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

from .cli import execute_cmd_with_json_output


def list_tasks(cluster, dir=None):
    return execute_cmd_with_json_output(
        ["aws", "ecs", "list-tasks", "--cluster", cluster, "--no-paginate"],
        wdir=dir,
        err=f"could not get the list of ecs tasks for cluster {cluster}",
    )


def describe_tasks(cluster, task, dir=None):
    return execute_cmd_with_json_output(
        ["aws", "ecs", "describe-tasks", "--cluster", cluster, "--task", task],
        wdir=dir,
        err=f"could not get details of task {task}",
    )
