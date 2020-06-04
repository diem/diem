#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

import json
import random
import re
import sys
import time
from pyhelpers.cli import execute_cmd_with_json_output
from pyhelpers.s3_lock import _get_lock, try_lock, unlock, _lock_exists

# global values for compat-specific lock operations
COMPAT_LOCK_BUCKET = "libra-compat-test-locks"
COMPAT_LOCK_PREFIX = "compat-lock-"
COMPAT_CLUSTER_PREFIX = "compat-"
TEMP_OBJ_PATH = "lock_obj.txt"


def get_task_defs(workspace: str) -> dict:
    """
    Builds a dictionary of:
        family -> current_task_def

    task_def can be used to get the following when updating a service:
        - containerDefinitions
        - volumes
        - placementConstraints

    NOTE: only possible to get the current running list of tasks, so tf apply needed after
          need a way to restore to steady state if an ecs update makes node boot loop
    """
    ret = {}
    print("Fetching ECS tasks")
    ecs_tasks = execute_cmd_with_json_output(
        ["aws", "ecs", "list-tasks", "--cluster", workspace, "--no-paginate"],
        err="could not get the list of ecs tasks",
    )
    for task_arn in ecs_tasks.get("taskArns"):
        tasks = execute_cmd_with_json_output(
            [
                "aws",
                "ecs",
                "describe-tasks",
                "--cluster",
                workspace,
                "--task",
                task_arn,
            ],
            err=f"could not get details of task {task_arn}",
        )
        task_def_arn = tasks.get("tasks")[0].get("taskDefinitionArn")
        if "validator" not in task_def_arn and "fullnode" not in task_def_arn:
            print(f"Skipping task {task_def_arn}")
            continue

        print(f"Fetching task definition for {task_def_arn}")
        task_def = execute_cmd_with_json_output(
            [
                "aws",
                "ecs",
                "describe-task-definition",
                "--task-definition",
                task_def_arn,
            ],
            err=f"could not get task definition for {task_def_arn}",
        )

        key = task_def.get("taskDefinition").get("family")
        ret[key] = task_def.get("taskDefinition")
        # put the tags separately
        tags = task_def.get("tags")
        ret[key]["tags"] = tags if tags else []

    print()
    return ret


def update_task_def_image_tags(task_def, new_tag):
    """
    Returns a new task_def that has been updated with the new tag
    """
    def_str = json.dumps(task_def)
    try:
        def_str = re.sub(
            r"libra_validator:[_0-9a-zA-Z]*", f"libra_validator:{new_tag}", def_str
        )
        def_str = re.sub(
            r"libra_safety_rules:[_0-9a-zA-Z]*",
            f"libra_safety_rules:{new_tag}",
            def_str,
        )
        return json.loads(def_str)
    except:
        print(f"ERROR: unable to parse and replace libra_validator image")
        raise


def get_image_tag_by_task_def(task_def):
    """
    Find libra_validator image tag or error
    """
    def_str = json.dumps(task_def)
    try:
        return re.search(r"libra_validator:[_0-9a-zA-Z]*", def_str)[0].split(":")[1]
    except:
        print(f"ERROR: unable to parse libra_validator image")
        raise


def _verify_cluster_locks(pool_size: int):
    """Verify that all clusters have locks, failing if locks are missing"""
    for cluster_idx in range(pool_size):
        cluster_lock = COMPAT_LOCK_PREFIX + str(cluster_idx)
        obj = _get_lock(cluster_lock, COMPAT_LOCK_BUCKET, TEMP_OBJ_PATH)
        # if the object doesn't exist, report it and exit
        # creating it now might result in race condition
        if not obj:
            print(
                f"Could not get cluster lock {cluster_lock} from bucket {COMPAT_LOCK_BUCKET}. Exiting..."
            )
            sys.exit(1)


def unlock_cluster(cluster_lock: str) -> str:
    """Unlock a cluster lock by name"""
    if not _lock_exists(cluster_lock, COMPAT_LOCK_BUCKET):
        print(f"ERROR: Lock {cluster_lock} not found")
        sys.exit(1)

    success = unlock(cluster_lock, COMPAT_LOCK_BUCKET)
    if not success:
        print(f"ERROR: failed to unlock {cluster_lock}")
        sys.exit(1)


def select_compat_cluster(pool_size: int, content: str) -> str:
    """
    Spin to select a free cluster, writing content to its lock. Returns lock name (lock's s3 object key),
    or sys exit due to missing lock or time out.
    """

    # verify that all clusters have their own lock
    _verify_cluster_locks(pool_size)

    clusters = list(range(pool_size))
    for attempt in range(360):
        random.shuffle(clusters)
        for cluster_idx in clusters:
            cluster_lock = COMPAT_LOCK_PREFIX + str(cluster_idx)
            cluster_workspace = COMPAT_CLUSTER_PREFIX + str(cluster_idx)

            if try_lock(cluster_lock, COMPAT_LOCK_BUCKET, TEMP_OBJ_PATH, content,):
                print(f"------------ Acquired {cluster_workspace} ------------\n")
                return cluster_workspace

        print(
            f"Attempt {attempt}: All clusters have jobs running on them. Retrying in 10 secs."
        )
        time.sleep(10)
    print("Failed to schedule job on a cluster as all are busy")
    sys.exit(1)
