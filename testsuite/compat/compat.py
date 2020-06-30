#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

#
# Tool for spinning up and upgrading long-running mixed validator version networks.
# Gets a cluster via lock, upgrade validator revisions, run a workload, unlock cluster
# Assumptions:
#     - ECS cluster pool named compat-[0-9]*
#     - Clusters run non-persistent storage
#     - Upgrade validators 0,1 for stateful tests, as they will state-sync with their fullnodes
#

import argparse
import os
import sys
import time
from pyhelpers.cli import execute_cmd_with_json_output, execute_cmd_with_text_output
from pyhelpers.aws import (
    update_service_task_def,
    register_task_def_update,
    update_service_force,
    batch_image_exists,
)
from pyhelpers.git import latest_testnet_tag, prev_tag
from compat_helpers import (
    get_task_defs,
    update_task_def_image_tags,
    get_image_tag_by_task_def,
    select_compat_cluster,
    unlock_cluster,
)

# TODO: currently unused, https://github.com/libra/libra/issues/4765
TESTNET_COMPATIBLE = int(os.getenv("TESTNET_COMPATIBLE", 1))
PREV_COMPATIBLE = int(os.getenv("PREV_COMPATIBLE", 1))
ECS_POOL_SIZE = int(os.getenv("ECS_POOL_SIZE", 1))
NUM_VALIDATORS = 4
NUM_FULLNODES = 2

parser = argparse.ArgumentParser(
    description="Upgrades validator and fullnodes in a compat cluster"
)
subparsers = parser.add_subparsers(required=True, title="commands", dest="command")

# command: compat admin
admin_parser = subparsers.add_parser(
    "admin", help="administrative functions to manager cluster state"
)
admin_mode_subparser = admin_parser.add_subparsers(
    required=True, title="admin mode commands", dest="admin_mode"
)
# command: compat admin unlock <cluster>
unlock_parser = admin_mode_subparser.add_parser(
    "unlock", help="unlock a specific cluster"
)
unlock_parser.add_argument("cluster_lock", help="cluster lock to unlock")
# command: compat admin reset <cluster>
reset_parser = admin_mode_subparser.add_parser(
    "reset", help="cluster to reset to initial config"
)
reset_parser.add_argument("cluster", help="cluster to reset")

# command: compat test --tag <tag> [--dry] [--workspace <personal_workspace>]
test_parser = subparsers.add_parser("test", help="test configuration options")
test_parser.add_argument(
    "--tag", "-t", required=True, help="image tag to test, default to HEAD"
)
test_parser.add_argument(
    "--dry", action="store_true", help="dry run without upgrading the cluster"
)
test_parser.add_argument(
    "--workspace", "-w", help="workspace to upgrade, overriding auto-selection",
)
test_mode_subparser = test_parser.add_subparsers(
    required=True, title="test mode commands", dest="test_mode"
)
# command: compat test --tag <tag> testnet
testnet_parser = test_mode_subparser.add_parser(
    "testnet", help="run compat test against testnet"
)
# command: compat test --tag <tag> manual --alt-tag <alt_tag>
manual_parser = test_mode_subparser.add_parser(
    "manual", help="run compat test against manually specified revision"
)
manual_parser.add_argument(
    "--alt-tag",
    required=True,
    help="alternative image tag to test against specified tag, for debugging",
)
args = parser.parse_args()

COMMAND = args.command
ADMIN_MODE = args.admin_mode if "admin_mode" in args.__dict__ else None
TEST_MODE = args.test_mode if "test_mode" in args.__dict__ else None

# parse admin
if COMMAND == "admin":
    if ADMIN_MODE == "unlock":
        unlock_cluster(args.cluster_lock)
        print(f"Successfully unlocked {args.cluster_lock}!")
    elif ADMIN_MODE == "reset":
        for i in range(NUM_VALIDATORS):
            update_service_task_def(args.cluster, f"{args.cluster}-validator-{i}", 1)
        for i in range(NUM_FULLNODES):
            update_service_task_def(args.cluster, f"{args.cluster}-fullnode-{i}", 1)
    sys.exit(0)


print("Starting compat test with the following configuration:")
print(f"\tTESTNET_COMPATIBLE={TESTNET_COMPATIBLE}")
print(f"\tPREV_COMPATIBLE={PREV_COMPATIBLE}")
print(f"\tECS_POOL_SIZE={ECS_POOL_SIZE}")

# get the right tags for testing
TEST_TAG = args.tag
if TEST_MODE == "testnet":
    ALT_TEST_TAG = latest_testnet_tag()
else:
    ALT_TEST_TAG = args.alt_tag

if not all(
    [
        batch_image_exists("libra_validator", [TEST_TAG, ALT_TEST_TAG]),
        batch_image_exists("libra_safety_rules", [TEST_TAG, ALT_TEST_TAG]),
    ]
):
    print("Missing images, exiting...")
    sys.exit(1)

if args.workspace:
    WORKSPACE = args.workspace
elif args.dry:
    print("Dry run requires explicit workspace -w")
    sys.exit(1)
else:
    WORKSPACE = select_compat_cluster(ECS_POOL_SIZE, TEST_TAG)
    # fails if no workspace

def_map = get_task_defs(WORKSPACE, NUM_VALIDATORS, NUM_FULLNODES)

# get the task definition families as they are the def_map keys
v_fams = [f"{WORKSPACE}-validator-{i}" for i in range(NUM_VALIDATORS)]
f_fams = [f"{WORKSPACE}-fullnode-{i}" for i in range(NUM_FULLNODES)]
vault_fam = f"{WORKSPACE}-vault"

# all tasks in the cluster are running
try:
    fams = v_fams + f_fams
    for fam in fams:
        assert fam in def_map
except Exception as e:
    print(f"ERROR: missing tasks in {WORKSPACE}, {e}")
    print("Retry or reset workspace: ./run_compat admin --reset ")
    sys.exit(1)

# get the current task definitions
v_defs = [def_map.get(fam) for fam in v_fams]
f_defs = [def_map.get(fam) for fam in f_fams]

# get the current image tags
curr_v_tags = [get_image_tag_by_task_def(v_def) for v_def in v_defs]
curr_f_tags = [get_image_tag_by_task_def(f_def) for f_def in f_defs]

print("Network is currently running the following versions:")
for i in range(NUM_VALIDATORS):
    print(f"\t{v_fams[i]} = {curr_v_tags[i]}")
for i in range(NUM_FULLNODES):
    print(f"\t{f_fams[i]} = {curr_f_tags[i]}")
print()


if TEST_MODE == "manual":
    print("\n------------ Starting MANUAL compat test ------------\n")
    print("Upgrade path:")
    print("\tvault (force)\t\t==> latest")
    print(f"\tvalidators {list(range(NUM_VALIDATORS // 2))} \t==> {ALT_TEST_TAG}")
    print(f"\tvalidators {list(range(NUM_VALIDATORS // 2, NUM_VALIDATORS))}\t==> {TEST_TAG}")
    print(f"\tfullnodes {list(range(NUM_FULLNODES))}\t==> {ALT_TEST_TAG}\n")
    new_v_defs = [
        update_task_def_image_tags(v_def, ALT_TEST_TAG)
        for v_def in v_defs[: NUM_VALIDATORS // 2]
    ]
    new_v_defs += [
        update_task_def_image_tags(v_def, TEST_TAG)
        for v_def in v_defs[NUM_VALIDATORS // 2 :]
    ]
    new_f_defs = [
        update_task_def_image_tags(f_def, ALT_TEST_TAG)
        for f_def in f_defs[: NUM_VALIDATORS // 2]
    ]

    if args.dry:
        print(f"DRY RUN: updated validators/fullnodes {list(range(NUM_VALIDATORS // 2))} to {ALT_TEST_TAG}")
        print(f"DRY RUN: updated validators {list(range(NUM_VALIDATORS // 2, NUM_VALIDATORS))} to {TEST_TAG}")
    else:
        v_revs = [
            register_task_def_update(v_fams[i], new_v_defs[i])
            for i in range(NUM_VALIDATORS)
        ]
        f_revs = [
            register_task_def_update(f_fams[i], new_f_defs[i])
            for i in range(NUM_FULLNODES)
        ]

        update_service_force(WORKSPACE, vault_fam)

        for i in range(NUM_VALIDATORS):
            update_service_task_def(WORKSPACE, v_fams[i], v_revs[i])

        print("Wait 30 seconds before updating fullnodes.")
        time.sleep(30)

        for i in range(NUM_FULLNODES):
            update_service_task_def(WORKSPACE, f_fams[i], f_revs[i])

elif TEST_MODE == "testnet":
    print("\n------------ Starting TESTNET compat test ------------\n")
    print("Upgrade path:")
    print("\tvault (force)\t\t==> latest")
    print(f"\tvalidators {list(range(NUM_VALIDATORS // 2))}\t==> {TEST_TAG}")
    print(f"\tvalidators {list(range(NUM_VALIDATORS // 2, NUM_VALIDATORS))}\t==> {ALT_TEST_TAG}")
    print(f"\tfullnodes {list(range(NUM_FULLNODES))}\t==> {ALT_TEST_TAG}\n")
    print("Checking if nodes are already running latest testnet version...")
    wrong_versions = False
    all_tags = curr_v_tags + curr_f_tags
    if not all([tag == ALT_TEST_TAG for tag in all_tags]):
        wrong_versions = True

    if wrong_versions:
        print(
            f"Cluster is not running testnet version, patching entire network to TESTNET_TAG = {ALT_TEST_TAG} and exiting..."
        )
        # create the patch definitions
        patch_v_defs = [
            update_task_def_image_tags(v_def, ALT_TEST_TAG) for v_def in v_defs
        ]
        patch_f_defs = [
            update_task_def_image_tags(f_def, ALT_TEST_TAG) for f_def in f_defs
        ]

        if not args.dry:

            # register the task defintions
            v_revs = [
                register_task_def_update(v_fams[i], patch_v_defs[i])
                for i in range(NUM_VALIDATORS)
            ]
            f_revs = [
                register_task_def_update(f_fams[i], patch_f_defs[i])
                for i in range(NUM_FULLNODES)
            ]

            update_service_force(WORKSPACE, vault_fam)

            # update each of the ecs services
            for i in range(NUM_VALIDATORS):
                update_service_task_def(WORKSPACE, v_fams[i], v_revs[i])

            print("Wait 30 seconds before updating fullnodes.")
            time.sleep(30)

            for i in range(NUM_FULLNODES):
                update_service_task_def(WORKSPACE, f_fams[i], f_revs[i])

        print("Please retry when the network is stable running testnet version\n")
        sys.exit(1)

    # after this point, validators/fullnodes 2,3 are running what they should
    # time to upgrade validators 0,1
    print("Cluster is running testnet verion!\n")
    new_v_defs = [
        update_task_def_image_tags(v_def, TEST_TAG)
        for v_def in v_defs[: NUM_VALIDATORS // 2]
    ]

    if not args.dry:
        v_revs = [
            register_task_def_update(v_fams[i], new_v_defs[i])
            for i in range(NUM_VALIDATORS // 2)
        ]

        for i in range(NUM_VALIDATORS // 2):
            update_service_task_def(WORKSPACE, v_fams[i], v_revs[i])

    else:
        print(f"DRY RUN: updated validators {list(range(NUM_VALIDATORS // 2))} to {TEST_TAG}")

print("Done!")
