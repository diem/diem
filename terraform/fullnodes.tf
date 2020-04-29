locals {
  total_num_fullnodes = var.num_fullnodes * var.num_fullnode_networks
  fullnode_command    = var.log_to_file ? jsonencode(["bash", "-c", "/docker-run-dynamic-fullnode.sh >> /opt/libra/data/libra.log 2>&1"]) : jsonencode(["bash", "-c", "/docker-run-dynamic-fullnode.sh"])
}

resource "aws_instance" "fullnode" {
  count         = local.total_num_fullnodes
  ami           = local.aws_ecs_ami
  instance_type = var.validator_type
  subnet_id = element(
    aws_subnet.testnet.*.id,
    count.index % length(data.aws_availability_zones.available.names),
  )
  depends_on                  = [aws_main_route_table_association.testnet, aws_iam_role_policy.ecs_extra]
  vpc_security_group_ids      = [aws_security_group.validator.id]
  private_ip                  = length(var.override_fullnode_ips) == 0 ? null : var.override_fullnode_ips[count.index]
  associate_public_ip_address = local.instance_public_ip
  key_name                    = aws_key_pair.libra.key_name
  iam_instance_profile        = aws_iam_instance_profile.ecsInstanceRole.name
  user_data                   = local.user_data

  dynamic "root_block_device" {
    for_each = contains(local.ebs_types, split(".", var.validator_type)[0]) ? [0] : []
    content {
      volume_type = "io1"
      volume_size = var.validator_ebs_size
      iops        = var.validator_ebs_size * 50 # max 50iops/gb
    }
  }

  tags = {
    Name          = "${terraform.workspace}-fullnode-${count.index}"
    Role          = "fullnode"
    Terraform     = "testnet"
    Workspace     = terraform.workspace
    FullnodeIndex = count.index
  }

}


data "template_file" "fullnode_ecs_task_definition" {
  count    = local.total_num_fullnodes
  template = file("templates/fullnode.json")

  vars = {
    image         = local.image_repo
    image_version = local.image_version
    cpu           = local.cpu_by_instance[var.validator_type]
    mem           = local.mem_by_instance[var.validator_type]

    cfg_listen_addr    = element(aws_instance.fullnode.*.private_ip, count.index)
    cfg_num_validators = var.cfg_num_validators_override == 0 ? var.num_validators : var.cfg_num_validators_override
    cfg_seed           = var.config_seed

    cfg_seed_peer_ip   = element(aws_instance.validator.*.private_ip, count.index % var.num_fullnode_networks)
    cfg_fullnode_index = (count.index % var.num_fullnodes)
    cfg_num_fullnodes  = var.num_fullnodes
    cfg_fullnode_seed  = var.fullnode_seed

    fullnode_id  = count.index
    log_level    = var.validator_log_level
    log_group    = var.cloudwatch_logs ? aws_cloudwatch_log_group.testnet.name : ""
    log_region   = var.region
    log_prefix   = "fullnode-${count.index}"
    capabilities = jsonencode(var.validator_linux_capabilities)
    command      = local.fullnode_command
  }
}

resource "aws_ecs_task_definition" "fullnode" {
  count  = local.total_num_fullnodes
  family = "${terraform.workspace}-fullnode-${count.index}"
  container_definitions = element(
    data.template_file.fullnode_ecs_task_definition.*.rendered,
    count.index,
  )
  execution_role_arn = aws_iam_role.ecsTaskExecutionRole.arn
  network_mode       = "host"

  volume {
    name      = "libra-data"
    host_path = var.persist_libra_data ? "/data/libra" : ""
  }

  placement_constraints {
    type       = "memberOf"
    expression = "ec2InstanceId == ${element(aws_instance.fullnode.*.id, count.index)}"
  }

  tags = {
    FullnodeId = count.index
    Role       = "fullnode"
    Terraform  = "testnet"
    Workspace  = terraform.workspace
  }
}

resource "aws_ecs_service" "fullnode" {
  count                              = local.total_num_fullnodes
  name                               = "${terraform.workspace}-fullnode-${count.index}"
  cluster                            = aws_ecs_cluster.testnet.id
  task_definition                    = element(aws_ecs_task_definition.fullnode.*.arn, count.index)
  desired_count                      = 1
  deployment_minimum_healthy_percent = 0

  tags = {
    FullnodeId = count.index
    Role       = "fullnode"
    Terraform  = "testnet"
    Workspace  = terraform.workspace
  }
}
