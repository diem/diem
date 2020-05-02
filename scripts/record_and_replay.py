#!/usr/bin/env python3
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Records state from cluster on existing terraform workspace and creates tfvars file
# for replay elsewhere


import argparse
import json
import sys
import os
from subprocess import check_call, check_output, DEVNULL, CalledProcessError


def gen_tf_var_str(var_name: str, val: str) -> str:
    return f'{var_name} = "{val}"'


def gen_tf_var_list(var_name: str, vals: list) -> str:
    list_str = ','.join(f'"{v}"' for v in vals)
    return f'{var_name} = [{list_str}]'


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
    print(
        f"ERROR: Could not find source workspace \"{args.source}\". Is it initialized?")
    sys.exit(1)

print(f"Using source workspace \"{args.source}\"")


# get source state with terraform show
try:
    out_raw = check_output(['terraform', 'show', '-json'], cwd=tf_dir, stderr=DEVNULL)
    out = json.loads(out_raw)
except CalledProcessError:
    print("ERROR: could not read info from terraform")
    sys.exit(1)


# get resources of importance
validators = []
fullnodes = []
validators_ecs = []
fullnodes_ecs = []
for res in out.get('values').get('root_module').get('resources'):
    addr = res.get('address')
    if 'aws_instance.validator' in addr:
        validators.append(res)
    elif 'aws_instance.fullnode' in addr:
        fullnodes.append(res)
    elif 'ecs_task_definition.validator' in addr:
        validators_ecs.append(res)
    elif 'ecs_task_definition.fullnode' in addr:
        fullnodes_ecs.append(res)


# terraform show not guaranteed to be sorted
list.sort(validators, key=lambda x: x.get('index'))
list.sort(fullnodes, key=lambda x: x.get('index'))
list.sort(validators_ecs, key=lambda x: x.get('index'))
list.sort(fullnodes_ecs, key=lambda x: x.get('index'))


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
for ecs in validators_ecs:
    containers = json.loads(ecs.get('values').get('container_definitions'))
    for container in containers:
        container_name = container.get('name')
        try:
            image, version = container.get('image').split(':')
        except:
            print("Error getting ECS images")
            sys.exit(1)
        if container_name == 'logstash':
            logstash_image = image
            logstash_version = version
        elif container_name == 'validator':
            validator_image = image
            validator_versions.append(version)
        elif container_name == 'safety-rules':
            safetyrules_image = image
            safetyrules_version = version

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

print()
print(f"cd {tf_dir}")
print("terraform workspace new <new_workspace>")
print(f"terraform apply --var-file={out_file}")
