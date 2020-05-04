#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Records state from cluster on existing terraform workspace, and replays elsewhere
# Only node IP and volumes are necessary for replay
# Image tags can be specified in existing tfvars files

# given an array, formats to a terraform -var '<content>' string
gen_tf_var_str() {
    local var_name="${1}"
    # local joined=$(join_by "," "${@:2}")
    local joined=$(printf ',"%s"' "${@:2}")
    echo "-var ${var_name}=[${joined:1}]"
}

join_by() {
    joined=$(printf ",'%s'" "${arr[@]}")
    IFS="$1"
    shift
    echo "$*"
}

while (("$#")); do
    case "$1" in
    --source)
        source_workspace=$2
        shift 2
        ;;
    --target)
        target_workspace=$2
        shift 2
        ;;
    --tf-dir)
        tf_dir=$2
        shift 2
        ;;
    --var-file)
        var_file=$2
        shift 2
        ;;
    --dry-run)
        echo "Executing dry run. No changes to Terraform will be made."
        dry_run=1
        shift 1
        ;;
    --verbose)
        verbose=1
        shift 1
        ;;
    *)
        break
        ;;
    esac
done

if [ -z "$source_workspace" ] || [ -z "$target_workspace" ] || [ -z "$var_file" ]; then
    echo "Missing argument"
    echo "Usage:"
    echo "record_and_replay --source <source workspace> --target <target workspace> --var-file <terraform tfvars file> [--dry-run] [--verbose]"
    echo
    echo "Examples:"
    echo "record_and_replay --source dev --target devtest --var-file dev.tfvars # replay dev network to new devtest workspace"
    exit 1
fi

# get to terraform dir
SCRIPT_PATH="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
if [ -z "$tf_dir" ]; then
    cd "$SCRIPT_PATH/../terraform"
else
    cd $tf_dir || exit 1
fi

echo -e "Searching workspaces in $PWD...\n"

terraform workspace select $source_workspace >/dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "Using source workspace \"$source_workspace\""
else
    echo "ERROR: Could not find workspace \"$source_workspace\". Is it initialized?"
    exit 1
fi

terraform workspace select $target_workspace >/dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "Using target workspace \"$target_workspace\""
else
    echo "Could not find workspace \"$target_workspace\". Would you like to initialize a new workspace?"
    echo -e "Only \"yes\" will be accepted to confirm.\n"
    echo "Enter a value: "
    read confirmation
    if [ "$confirmation" = "yes" ]; then
        if [ -z "$dry_run" ]; then
            echo "EXECUTE: terraform workspace new $target_workspace"
            terraform workspace new $target_workspace ｜｜ exit 1
        else
            echo "DRY-EXECUTE: terraform workspace new $target_workspace"
            echo
        fi
    else
        echo "Record and replay canceled"
        exit 1
    fi
fi

# done fetching terraform workspaces
# now get aws data
echo

######
# Fetch validator info (restore EBS volumes and private IPs) from aws and transform them into terraform vars
# NOTE: overrides IFS to maintain tab and newline formatting from aws cli
######

OLDIFS=$IFS

IFS="" # set IFS to retain formatting of aws cli output
validators_description="$(aws ec2 describe-instances --filter "Name=tag:Workspace,Values=$source_workspace" "Name=tag:Role,Values=validator" --query "Reservations[*].Instances[*].[BlockDeviceMappings,PrivateIpAddress,Tags[*]]" --output text)"
if [ $? -ne 0 ]; then
    echo "aws query for validators failed. Have you MFA'd today?"
    exit 1
fi

# organize the data into table
validators_inst_sorted=$(echo $validators_description | grep -A 1 -E "/dev/xvdb|NodeIndex|([0-9]{1,3}[\.]){3}[0-9]{1,3}" | grep -E "vol-|NodeIndex|([0-9]{1,3}[\.]){3}[0-9]{1,3}" | awk '/([0-9]{1,3}[\.]){3}[0-9]{1,3}/ {ip=$0}/^EBS/ {s=$0} /^NodeIndex/ {print ip "\t" s "\t" $0}' | sort -k 7)

# after which point, validators_inst_sorted should look like this, sorted by index:
#
# 10.0.2.184      EBS     2020-04-13T18:07:31+00:00       True    attached        vol-0d4a0798634333cc5   NodeIndex       0
# 10.0.5.78       EBS     2020-04-13T18:07:39+00:00       True    attached        vol-0de6aa88534db9917   NodeIndex       1
# ...

if [ -n "$verbose" ]; then
    echo $validators_inst_sorted
    echo
fi

vols_sorted=($(echo $validators_inst_sorted | awk '{ print $6 }'))

IFS=$OLDIFS
var_str_restore_vols=$(gen_tf_var_str "restore_vol_ids" $vols_sorted)

IFS="" # set IFS to retain formatting of aws cli output
validators_ips_sorted=($(echo $validators_inst_sorted | awk '{ print $1 }'))

IFS=$OLDIFS
var_str_validator_ips=$(gen_tf_var_str "override_validator_ips" $validators_ips_sorted)

######
# Fetch fullnode info (private IPs) from aws and transform them into terraform vars
# NOTE: overrides IFS to maintain tab and newline formatting from aws cli
######

IFS=
fullnodes_description=$(aws ec2 describe-instances --filter "Name=tag:Workspace,Values=$source_workspace" "Name=tag:Role,Values=fullnode" --query "Reservations[*].Instances[*].[PrivateIpAddress,Tags[*]]" --output text)
if [ $? -ne 0 ]; then
    echo "aws query for fullnodes failed. Have you MFA'd today?"
    exit 1
fi

# organize the data into a table
fullnodes_inst_sorted=$(echo $fullnodes_description | grep -E "FullnodeIndex|([0-9]{1,3}[\.]){3}[0-9]{1,3}" | awk '/([0-9]{1,3}[\.]){3}[0-9]{1,3}/ {ip=$0} /^FullnodeIndex/ {print ip "\t" $0}' | sort -k 4)

# after which point, fullnodes_inst_sorted should look like this, sorted by index:
#
# 10.0.1.170      FullnodeIndex   0
# 10.0.6.208      FullnodeIndex   1
# ...

if [ -n "$verbose" ]; then
    echo $fullnodes_inst_sorted
    echo
fi

fullnodes_ips_sorted=($(echo $fullnodes_inst_sorted | awk '{ print $1 }'))

IFS=$OLDIFS
var_str_fullnode_ips=$(gen_tf_var_str "override_fullnode_ips" $fullnodes_ips_sorted)

# hand over control to terraform
if [ -z "$dry_run" ]; then
    echo "EXECUTE: terraform apply $var_str_restore_vols $var_str_validator_ips $var_str_fullnode_ips --var-file $var_file"
    terraform apply $var_str_restore_vols $var_str_validator_ips $var_str_fullnode_ips --var-file $var_file
else
    echo "DRY-EXECUTE: terraform apply $var_str_restore_vols $var_str_validator_ips $var_str_fullnode_ips --var-file $var_file"
    echo
fi

if [ $? -eq 0 ]; then
    echo "Record and Replay successful!"
fi
