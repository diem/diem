#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

import json
from subprocess import CalledProcessError
from pyhelpers import aws


def test_batch_image_exists(monkeypatch) -> None:
    mock_ret = {
        "images": [
            {
                "registryId": "123456789012",
                "repositoryName": "MyRepository",
                "imageId": {"imageDigest": "sha256:123456789012", "imageTag": "abc"},
                "imageManifest": "manifestExample1",
            },
            {
                "registryId": "567890121234",
                "repositoryName": "MyRepository",
                "imageId": {"imageDigest": "sha256:123456789012", "imageTag": "xyz"},
                "imageManifest": "manifestExample2",
            },
        ],
        "failures": [],
    }
    monkeypatch.setattr(
        "pyhelpers.cli.check_output", lambda *args, **kwargs: json.dumps(mock_ret)
    )
    assert aws.batch_image_exists("abc", ["xyz"])

    mock_ret = {
        "images": [],
        "failures": [
            {
                "imageId": {"imageTag": "abc"},
                "failureCode": "ImageNotFound",
                "failureReason": "Requested image not found",
            },
            {
                "imageId": {"imageTag": "xyz"},
                "failureCode": "ImageNotFound",
                "failureReason": "Requested image not found",
            },
        ],
    }
    monkeypatch.setattr(
        "pyhelpers.cli.check_output", lambda *args, **kwargs: json.dumps(mock_ret)
    )
    assert not aws.batch_image_exists("abc", ["xyz"])

    mock_ret = {
        "images": [
            {
                "registryId": "123456789012",
                "repositoryName": "MyRepository",
                "imageId": {"imageDigest": "sha256:123456789012", "imageTag": "abc"},
                "imageManifest": "manifestExample1",
            },
        ],
        "failures": [
            {
                "imageId": {"imageTag": "xyz"},
                "failureCode": "ImageNotFound",
                "failureReason": "Requested image not found",
            },
        ],
    }
    monkeypatch.setattr(
        "pyhelpers.cli.check_output", lambda *args, **kwargs: json.dumps(mock_ret)
    )
    assert not aws.batch_image_exists("abc", ["xyz"])
