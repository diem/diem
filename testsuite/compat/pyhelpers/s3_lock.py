#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
import hashlib
import base64
from datetime import datetime, timedelta
from .cli import execute_cmd_with_json_output, execute_cmd_with_text_output

TIMESTAMP_FMT = "%Y-%m-%dT%H:%M:%S.%fZ"
RETENTION_BYPASS = "Mode=GOVERNANCE,RetainUntilDate="
DEFL_RETENTION_LENGTH = 60 * 60 * 3  # processes hold locks for 3 hrs


def try_lock(cluster_lock: str, bucket: str, path: str, contents: str) -> bool:
    """Try to obtain the cluster lock and overwrite its retention"""
    obj_output = _get_lock(cluster_lock, bucket, path)
    if not obj_output or "ObjectLockRetainUntilDate" not in obj_output:
        print(f"Something's not right with lock {cluster_lock}, skipping...")
        return False

    expiration_str = obj_output.get("ObjectLockRetainUntilDate")
    expiration_date = datetime.strptime(expiration_str, TIMESTAMP_FMT)
    now = datetime.utcnow()
    if now < expiration_date:
        print(
            f"Lock {cluster_lock} expires in {expiration_date - now} or upon job complete"
        )
        return False

    retention_date = _get_retention_date(DEFL_RETENTION_LENGTH)
    retention_str = _get_retention_str(retention_date)

    version = obj_output.get("VersionId")
    put_obj_output = _put_lock(cluster_lock, bucket, version, retention_str)
    if not put_obj_output:
        print(f"Failed to obtain lock {cluster_lock}")
        return False

    # To address race, get the lock object again and check if we were the ones who got it
    # by checking if the retention is exactly the same as what we set it (in ms)
    get_lock_output = _get_lock(cluster_lock, bucket, path)
    if not get_lock_output or "ObjectLockRetainUntilDate" not in get_lock_output:
        print(f"Something's not right with lock {cluster_lock}, skipping...")
        return False

    expiration_str = get_lock_output.get("ObjectLockRetainUntilDate")
    expiration_date = datetime.strptime(expiration_str, TIMESTAMP_FMT)
    if expiration_date != retention_date:
        print(f"Got retention expiration {expiration_str}, expected {retention_str}")
        return False

    return True


def unlock(cluster_lock: str, bucket: str) -> bool:
    """Release the cluster lock by resetting retention"""

    retention = _get_retention_bypass()
    put_retention_output = execute_cmd_with_text_output(
        [
            "aws",
            "s3api",
            "put-object-retention",
            "--bucket",
            bucket,
            "--key",
            cluster_lock,
            "--bypass-governance-retention",
            "--retention",
            retention,
        ],
        err=f"Error putting cluster lock {cluster_lock} to s3",
    )
    # Expect no output by default: https://awscli.amazonaws.com/v2/documentation/api/latest/reference/s3api/put-object-retention.html#examples
    if put_retention_output:
        print(f"Failed to release lock {cluster_lock}")
        return False

    return True


def _lock_exists(cluster_lock: str, bucket: str) -> bool:
    try:
        execute_cmd_with_text_output(
            ["aws", "s3api", "head-object", "--bucket", bucket, "--key", cluster_lock],
            err=f"Error getting lock {cluster_lock}",
        )
        return True
    except:
        return False


def _get_lock(cluster_lock: str, bucket: str, path: str) -> dict:
    """Returns the lock object from s3"""
    obj_output = execute_cmd_with_json_output(
        [
            "aws",
            "s3api",
            "get-object",
            "--bucket",
            bucket,
            "--key",
            cluster_lock,
            path,
        ],
        err=f"Error getting cluster lock {cluster_lock} from s3",
    )
    return obj_output


def _put_lock(cluster_lock: str, bucket: str, version: str, retention: str) -> bool:
    """Put the lock object to s3. Potentially fails due to retention"""
    put_retention_output = execute_cmd_with_text_output(
        [
            "aws",
            "s3api",
            "put-object-retention",
            "--bucket",
            bucket,
            "--key",
            cluster_lock,
            "--version-id",
            version,
            "--no-bypass-governance-retention",
            "--retention",
            retention,
        ],
        err=f"Error putting cluster lock {cluster_lock} to s3",
    )
    # Expect no output by default: https://awscli.amazonaws.com/v2/documentation/api/latest/reference/s3api/put-object-retention.html#examples
    if put_retention_output:
        print(f"Failed to release lock {cluster_lock}")
        return False

    return True


def _get_retention_bypass(offset_secs: int = 30) -> str:
    """
    Gets a retention config string offset from current time, bypassing the 1 day minimum.
    The 30s default retention bypass time is to address races, so we don't attempt to set
    retention time that's in the future later on.
    """
    retention_datetime = _get_retention_date(offset_secs)
    return _get_retention_str(retention_datetime)


def _get_retention_str(retention_datetime: datetime) -> str:
    now_str = datetime.strftime(retention_datetime, TIMESTAMP_FMT)
    return f"{RETENTION_BYPASS}{now_str}"


def _get_retention_date(offset_secs: int) -> datetime:
    """Get a datetime representing an offset from current time, rounded to nearest ms"""
    offset = timedelta(0, offset_secs)
    now = datetime.utcnow()
    rounded_us = now.microsecond // 1000 * 1000
    now = now.replace(microsecond=rounded_us)
    return now + offset


def _delete_lock(cluster_lock: str, version: str, bucket: str) -> None:
    """In the case that we have a race on acquiring the lock, remove it"""
    execute_cmd_with_json_output(
        [
            "aws",
            "s3api",
            "delete-object",
            "--bucket",
            bucket,
            "--key",
            cluster_lock,
            "--version-id",
            version,
            "--bypass-governance-retention",
        ],
        err=f"Error deleting cluster lock {cluster_lock}:{version}",
    )
