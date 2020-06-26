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

# TODO: currently unsed, https://github.com/libra/libra/issues/4765
TESTNET_COMPATIBLE = int(os.getenv("TESTNET_COMPATIBLE", 1))
PREV_COMPATIBLE = int(os.getenv("PREV_COMPATIBLE", 1))
ECS_POOL_SIZE = int(os.getenv("ECS_POOL_SIZE", 1))


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
        update_service_task_def(args.cluster, f"{args.cluster}-validator-0", 1)
        update_service_task_def(args.cluster, f"{args.cluster}-validator-1", 1)
        update_service_task_def(args.cluster, f"{args.cluster}-validator-2", 1)
        update_service_task_def(args.cluster, f"{args.cluster}-validator-3", 1)
        update_service_task_def(args.cluster, f"{args.cluster}-fullnode-0", 1)
        update_service_task_def(args.cluster, f"{args.cluster}-fullnode-1", 1)
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

def_map = get_task_defs(WORKSPACE)

# get the task definition families as they are the def_map keys
v0_fam = f"{WORKSPACE}-validator-0"
v1_fam = f"{WORKSPACE}-validator-1"
v2_fam = f"{WORKSPACE}-validator-2"
v3_fam = f"{WORKSPACE}-validator-3"
f0_fam = f"{WORKSPACE}-fullnode-0"
f1_fam = f"{WORKSPACE}-fullnode-1"
vault_fam = f"{WORKSPACE}-vault"

# all tasks in the cluster are running
try:
    assert v0_fam in def_map
    assert v1_fam in def_map
    assert v2_fam in def_map
    assert v3_fam in def_map
    assert f0_fam in def_map
    assert f1_fam in def_map
except Exception as e:
    print(f"ERROR: missing tasks in {WORKSPACE}, {e}")
    print("Retry or reset workspace: ./run_compat admin --reset ")
    sys.exit(1)

# get the current task definitions
v0_def = def_map.get(v0_fam)
v1_def = def_map.get(v1_fam)
v2_def = def_map.get(v2_fam)
v3_def = def_map.get(v3_fam)
f0_def = def_map.get(f0_fam)
f1_def = def_map.get(f1_fam)

# get the current image tags
curr_tag_v0 = get_image_tag_by_task_def(v0_def)
curr_tag_v1 = get_image_tag_by_task_def(v1_def)
curr_tag_v2 = get_image_tag_by_task_def(v2_def)
curr_tag_v3 = get_image_tag_by_task_def(v3_def)
curr_tag_f0 = get_image_tag_by_task_def(f0_def)
curr_tag_f1 = get_image_tag_by_task_def(f1_def)

print("Network is currently running the following versions:")
print(f"\t{v0_fam}={curr_tag_v0}")
print(f"\t{v1_fam}={curr_tag_v1}")
print(f"\t{v2_fam}={curr_tag_v2}")
print(f"\t{v3_fam}={curr_tag_v3}")
print(f"\t{f0_fam}={curr_tag_f0}")
print(f"\t{f1_fam}={curr_tag_f1}\n")


if TEST_MODE == "manual":
    print("\n------------ Starting MANUAL compat test ------------\n")
    print("Upgrade path:")
    print("\tvault (force)\t==> latest")
    print(f"\tvalidators 0, 1\t==> {ALT_TEST_TAG}")
    print(f"\tvalidators 2, 3\t==> {TEST_TAG}")
    print(f"\tfullnodes 0, 1\t==> {ALT_TEST_TAG}\n")
    new_v0_def = update_task_def_image_tags(v0_def, ALT_TEST_TAG)
    new_v1_def = update_task_def_image_tags(v1_def, ALT_TEST_TAG)
    new_f0_def = update_task_def_image_tags(f0_def, ALT_TEST_TAG)
    new_f1_def = update_task_def_image_tags(f1_def, ALT_TEST_TAG)
    new_v2_def = update_task_def_image_tags(v2_def, TEST_TAG)
    new_v3_def = update_task_def_image_tags(v3_def, TEST_TAG)

    if args.dry:
        print(f"DRY RUN: updated validators/fullnodes 0,1 to {ALT_TEST_TAG}")
        print(f"DRY RUN: updated validators 2,3 to {TEST_TAG}")
    else:
        rev_v0 = register_task_def_update(v0_fam, new_v0_def)
        rev_v1 = register_task_def_update(v1_fam, new_v1_def)
        rev_f0 = register_task_def_update(f0_fam, new_f0_def)
        rev_f1 = register_task_def_update(f1_fam, new_f1_def)
        rev_v2 = register_task_def_update(v2_fam, new_v2_def)
        rev_v3 = register_task_def_update(v3_fam, new_v3_def)

        update_service_force(WORKSPACE, vault_fam)

        update_service_task_def(WORKSPACE, v0_fam, rev_v0)
        update_service_task_def(WORKSPACE, v1_fam, rev_v1)
        update_service_task_def(WORKSPACE, v2_fam, rev_v2)
        update_service_task_def(WORKSPACE, v3_fam, rev_v3)

        print("Wait 30 seconds before updating fullnodes.")
        time.sleep(30)

        update_service_task_def(WORKSPACE, f0_fam, rev_f0)
        update_service_task_def(WORKSPACE, f1_fam, rev_f1)

elif TEST_MODE == "testnet":
    print("\n------------ Starting TESTNET compat test ------------\n")
    print("Upgrade path:")
    print("\tvault (force)\t==> latest")
    print(f"\tvalidators 0, 1\t==> {TEST_TAG}")
    print(f"\tvalidators 2, 3\t==> {ALT_TEST_TAG}")
    print(f"\tfullnodes 0, 1\t==> {ALT_TEST_TAG}\n")
    print("Diagnosing node health...")
    unhealthy = False
    all_tags = [
        curr_tag_v0,
        curr_tag_v1,
        curr_tag_v2,
        curr_tag_v3,
        curr_tag_f0,
        curr_tag_f1,
    ]
    if not all([tag == ALT_TEST_TAG for tag in all_tags]):
        unhealthy = True

    if unhealthy:
        print(
            f"Cluster is unhealthy, patching entire network to TESTNET_TAG = {ALT_TEST_TAG} and exiting..."
        )
        # create the patch definitions
        patch_v0_def = update_task_def_image_tags(v0_def, ALT_TEST_TAG)
        patch_v1_def = update_task_def_image_tags(v1_def, ALT_TEST_TAG)
        patch_v2_def = update_task_def_image_tags(v2_def, ALT_TEST_TAG)
        patch_v3_def = update_task_def_image_tags(v3_def, ALT_TEST_TAG)
        patch_f0_def = update_task_def_image_tags(f0_def, ALT_TEST_TAG)
        patch_f1_def = update_task_def_image_tags(f1_def, ALT_TEST_TAG)
        if not args.dry:

            # register the task defintions
            rev_v0 = register_task_def_update(v0_fam, patch_v0_def)
            rev_v1 = register_task_def_update(v1_fam, patch_v1_def)
            rev_v2 = register_task_def_update(v2_fam, patch_v2_def)
            rev_v3 = register_task_def_update(v3_fam, patch_v3_def)
            rev_f0 = register_task_def_update(f0_fam, patch_f0_def)
            rev_f1 = register_task_def_update(f1_fam, patch_f1_def)

            update_service_force(WORKSPACE, vault_fam)

            # update each of the ecs services
            update_service_task_def(WORKSPACE, v0_fam, rev_v0)
            update_service_task_def(WORKSPACE, v1_fam, rev_v1)
            update_service_task_def(WORKSPACE, v2_fam, rev_v2)
            update_service_task_def(WORKSPACE, v3_fam, rev_v3)

            print("Wait 30 seconds before updating fullnodes.")
            time.sleep(30)

            update_service_task_def(WORKSPACE, f0_fam, rev_f0)
            update_service_task_def(WORKSPACE, f1_fam, rev_f1)

        print("Please retry when the network is healthy\n")
        sys.exit(1)

    # after this point, validators/fullnodes 2,3 are running what they should
    # time to upgrade validators 0,1
    print("Cluster is healthy!\n")
    new_v0_def = update_task_def_image_tags(v0_def, TEST_TAG)
    new_v1_def = update_task_def_image_tags(v1_def, TEST_TAG)

    if not args.dry:
        rev_v0 = register_task_def_update(v0_fam, new_v0_def)
        rev_v1 = register_task_def_update(v1_fam, new_v1_def)

        update_service_task_def(WORKSPACE, v0_fam, rev_v0)
        update_service_task_def(WORKSPACE, v1_fam, rev_v1)
    else:
        print(f"DRY RUN: updated validators 0,1 to {TEST_TAG}")

print("Done!")
