#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

import json
from .cli import execute_cmd_with_json_output, execute_cmd_with_text_output


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


def update_service_force(cluster: str, family: str) -> None:
    """Force update a service"""
    execute_cmd_with_text_output(
        [
            "aws",
            "ecs",
            "update-service",
            "--force-new-deployment",
            "--cluster",
            cluster,
            "--service",
            family,
        ],
        err=f"Error force updating service {family}",
    )
    print(f"Updated service {family}")


def update_service_task_def(cluster: str, family: str, rev: str) -> None:
    """
    Assumes service name and task definition name are same as family name
    """
    execute_cmd_with_text_output(
        [
            "aws",
            "ecs",
            "update-service",
            "--no-force-new-deployment",
            "--cluster",
            cluster,
            "--service",
            family,
            "--task-definition",
            f"{family}:{rev}",
        ],
        err=f"Error updating service {family} to rev {rev}",
    )

    print(f"Updated service {family} to rev {rev}")


def register_task_def_update(family: str, task_def_obj: dict) -> str:
    """
    Registers a task definition to overwrite existing ones, publishing a new updated revision
    and returning its revision number
    """
    rev = execute_cmd_with_text_output(
        [
            "aws",
            "ecs",
            "register-task-definition",
            "--family",
            family,
            "--network-mode",
            task_def_obj.get("networkMode"),
            "--execution-role-arn",
            task_def_obj.get("executionRoleArn"),
            "--container-definitions",
            json.dumps(task_def_obj.get("containerDefinitions")),
            "--volumes",
            json.dumps(task_def_obj.get("volumes")),
            "--placement-constraints",
            json.dumps(task_def_obj.get("placementConstraints")),
            # '--tags', json.dumps(task_def_obj.get('tags')),
            "--output",
            "json",
            "--query",
            "taskDefinition.revision",
        ],
        err=f"Error registering new revision of {family}",
    )

    rev = rev.strip()

    print(f"Updated task definition {family} to revision {rev}")

    return rev


def batch_image_exists(repository_name: str, image_tags: list) -> bool:
    image_tags_strs = [f"imageTag={tag}" for tag in image_tags]
    get_image_output = execute_cmd_with_json_output(
        [
            "aws",
            "ecr",
            "batch-get-image",
            "--repository-name",
            repository_name,
            "--image-ids",
        ]
        + image_tags_strs,
        err=f"Error fetching {repository_name}:{image_tags}",
    )
    if not get_image_output:
        return False
    failures = get_image_output.get("failures")
    if len(failures) > 0:
        fail_tags = [fail.get("imageId").get("imageTag") for fail in failures]
        print(f"Image fetch failures {repository_name}:{fail_tags}")
        return False

    return True
