#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Records state from cluster on existing terraform workspace and creates tfvars file
# for replay elsewhere


import argparse
import json
import os
import re
from subprocess import check_call, check_output, DEVNULL, CalledProcessError


def gen_tf_var_str(var_name: str, val: str) -> str:
    return f'{var_name} = "{val}"'


def gen_tf_var_list(var_name: str, vals: list) -> str:
    list_str = ','.join(f'"{v}"' for v in vals)
    return f'{var_name} = [{list_str}]'

def execute_cmd_with_text_output(cmd: list, wdir: str, err: str) -> str:
    try:
        return check_output(cmd, cwd=wdir, stderr=DEVNULL)
    except CalledProcessError:
        print(f"ERROR: {err}")
        raise

def execute_cmd_with_json_output(cmd: list, wdir: str, err: str) -> dict:
    out = execute_cmd_with_text_output(cmd, wdir, err)
    return json.loads(out)

parser = argparse.ArgumentParser(
    description='Record state from a terraform workspace for replay in another.')

parser.add_argument('--source', '-s', required=True, help='source workspace')
parser.add_argument('--tf-dir', '-t', required=True,
                    help='terraform directory')

parser.add_argument('--var-file', '-f', help='terraform tfvars file to extend')
parser.add_argument('--out-file', '-o', help='name of output tfvars file')

args = parser.parse_args()


tf_dir = os.path.abspath(args.tf_dir)
print(f"Searching workspaces in {tf_dir}")

# check if source exists

try:
    check_call(['terraform', 'workspace', 'select', args.source], cwd=tf_dir, stdout=DEVNULL, stderr=DEVNULL)
except CalledProcessError:
    print(f"ERROR: Could not find source workspace \"{args.source}\". Is it initialized?")
    raise

print(f"Using source workspace \"{args.source}\"")

# get source state with terraform show
out = execute_cmd_with_json_output(
    ['terraform', 'show', '-json'],
    tf_dir,
    "could not read info from terraform"
)

# get resources of importance
validators = []
fullnodes = []
validators_ecs = []
for res in out.get('values').get('root_module').get('resources'):
    addr = res.get('address')
    if 'aws_instance.validator' in addr:
        validators.append(res)
    elif 'aws_instance.fullnode' in addr:
        fullnodes.append(res)
    elif 'ecs_task_definition.validator' in addr:
        validators_ecs.append(res)


# terraform show not guaranteed to be sorted
list.sort(validators, key=lambda x: x.get('index'))
list.sort(fullnodes, key=lambda x: x.get('index'))
list.sort(validators_ecs, key=lambda x: x.get('index'))


# parse validators
validator_ips = []
validator_restore_vols = []
for validator in validators:
    vals = validator.get('values')
    validator_ips.append(vals.get('private_ip'))
    for vol in vals.get('ebs_block_device'):
        if vol.get('device_name') == '/dev/xvdb':
            validator_restore_vols.append(vol.get('volume_id'))


# parse fullnodes
fullnode_ips = []
for fullnode in fullnodes:
    fullnode_ips.append(fullnode.get('values').get('private_ip'))


# Get image and version for ECS
logstash_image = None
validator_image = None
safetyrules_image = None
logstash_version = None
validator_versions = []
safetyrules_version = None
# Used to match the 'group' in task instance with 'family' in task definition. Why?
# The index isn't available in task instance but in task definition so we iterate
# over task definitions in order and use the mapping to extract the image and version
# from the corresponding task instance
validator_image_map = {}

ecs_tasks = execute_cmd_with_json_output(
    ['aws', 'ecs', 'list-tasks', '--cluster', args.source, '--no-paginate'],
    tf_dir,
    "could not get the list of ecs tasks"
)

for task in ecs_tasks.get('taskArns'):
    task_details = execute_cmd_with_json_output(
        ['aws', 'ecs', 'describe-tasks', '--cluster', args.source, '--task', task],
        tf_dir,
        f"could not get details of task {task}"
    )
    extract_image_tag = lambda container: (
        re.split('[@:]', container.get("image"))[0],
        container.get("imageDigest")
    )
    for container in task_details.get("tasks")[0].get("containers"):
        name = container.get("name")
        if not logstash_image and name == 'logstash':
            logstash_image, logstash_version = extract_image_tag(container)
        elif not safetyrules_image and name == 'safety-rules':
            safetyrules_image, safetyrules_version = extract_image_tag(container)
        elif name == 'validator':
            key = task_details.get("tasks")[0].get("group").split("service:",1)[1]
            validator_image_map[key] = extract_image_tag(container)

for ecs in validators_ecs:
    if ecs.get("name") == "validator":
        key = ecs.get("values").get("family")
        if key in validator_image_map:
            validator_image, version = validator_image_map[key]
            validator_versions.append(version)
        else:
            print(f"ERROR: didn't find task corresponding to definition {key} in {validator_image_map.keys()}")

print(f"logstash_image          : {logstash_image}")
print(f"logstash_version        : {logstash_version}")
print(f"safetyrules_image       : {safetyrules_image}")
print(f"safetyrules_version     : {safetyrules_version}")
print(f"validator_image         : {validator_image}")
print(f"validator_versions      : {validator_versions}")
print(f"validator_ips           : {validator_ips}")
print(f"fullnode_ips            : {fullnode_ips}")
print(f"validator_restore_vols  : {validator_restore_vols}")

# build the var strings
vars = []
vars.append(gen_tf_var_list('restore_vol_ids', validator_restore_vols))

vars.append(gen_tf_var_list('override_validator_ips', validator_ips))
vars.append(gen_tf_var_list('override_fullnode_ips', fullnode_ips))

vars.append(gen_tf_var_str('image_repo', validator_image))
vars.append(gen_tf_var_list('override_image_tags', validator_versions))
vars.append(gen_tf_var_str('logstash_image', logstash_image))
vars.append(gen_tf_var_str('logstash_version', logstash_version))
vars.append(gen_tf_var_str('safety_rules_image_repo', safetyrules_image))
vars.append(gen_tf_var_str('safety_rules_image_tag', safetyrules_version))

if args.var_file:
    with open(args.var_file, 'r') as f:
        source_varfile_text = f.read()

# write all vars to file
out_file = os.path.abspath(args.out_file) if args.out_file else 'roar.tfvars'
with open(out_file, 'w') as f:
    if args.var_file:
        f.write(source_varfile_text)
        f.write('\n')
        f.write(
            f'# Auto generated record and replay vars from file {os.path.abspath(args.var_file)}\n')
    else:
        f.write(
            f'# Auto generated record and replay vars inferred from workspace \"{args.source}\"\n')
    for var in vars:
        # avoid collision if var previously specified
        if args.var_file and var.split('=')[0] in source_varfile_text:
            continue
        f.write(var)
        f.write('\n')

print("\nInstructions:")
print(f"cd {tf_dir}")
print("terraform workspace new <new_workspace>")
print(f"terraform apply --var-file={out_file}")
