resource "aws_instance" "fullnode" {
  count         = var.num_fullnodes
  ami           = local.aws_ecs_ami
  instance_type = var.validator_type
  subnet_id = element(
    aws_subnet.testnet.*.id,
    count.index % length(data.aws_availability_zones.available.names),
  )
  depends_on                  = [aws_main_route_table_association.testnet, aws_iam_role_policy_attachment.ecs_extra]
  vpc_security_group_ids      = [aws_security_group.validator.id]
  associate_public_ip_address = local.instance_public_ip
  key_name                    = aws_key_pair.libra.key_name
  iam_instance_profile        = aws_iam_instance_profile.ecsInstanceRole.name
  user_data                   = local.user_data

  dynamic "root_block_device" {
    for_each = contains(local.ebs_types, split(var.validator_type, ".")[0]) ? [0] : []
    content {
      volume_type = "io1"
      volume_size = var.validator_ebs_size
      iops        = var.validator_ebs_size * 50 # max 50iops/gb
    }
  }

  tags = {
    Name          = "${terraform.workspace}-fullnode-${count.index}"
    Role          = "fullnode"
    Workspace     = terraform.workspace
    FullnodeIndex = count.index
  }

}


data "template_file" "fullnode_ecs_task_definition" {
  count    = var.num_fullnodes
  template = file("templates/validator.json")

  vars = {
    image            = local.image_repo
    image_version    = local.image_version
    cpu              = local.cpu_by_instance[var.validator_type]
    mem              = local.mem_by_instance[var.validator_type]
    fullnode_id      = count.index
    log_level        = var.validator_log_level
    log_group        = var.cloudwatch_logs ? aws_cloudwatch_log_group.testnet.name : ""
    log_region       = var.region
    log_prefix       = "fullnode-${count.index}"
    capabilities     = jsonencode(var.validator_linux_capabilities)
    command          = local.validator_command
  }
}

resource "aws_ecs_task_definition" "fullnode" {
  count  = var.num_fullnodes
  family = "${terraform.workspace}-fullnode-${count.index}"
  container_definitions = element(
    data.template_file.fullnode_ecs_task_definition.*.rendered,
    count.index,
  )
  execution_role_arn = aws_iam_role.ecsTaskExecutionRole.arn
  network_mode       = "host"

  volume {
    name      = "libra-data"
    host_path = "/data/libra"
  }

  placement_constraints {
    type       = "memberOf"
    expression = "ec2InstanceId == ${element(aws_instance.fullnode.*.id, count.index)}"
  }

  tags = {
    FullnodeId = count.index
    Role       = "fullnode"
    Workspace  = terraform.workspace
  }
}

resource "aws_ecs_service" "fullnode" {
  count                              = var.num_fullnodes
  name                               = "${terraform.workspace}-fullnode-${count.index}"
  cluster                            = aws_ecs_cluster.testnet.id
  task_definition                    = element(aws_ecs_task_definition.fullnode.*.arn, count.index)
  desired_count                      = 1
  deployment_minimum_healthy_percent = 0

  tags = {
    FullnodeId = count.index
    Role       = "fullnode"
    Workspace  = terraform.workspace
  }
}
