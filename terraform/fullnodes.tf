resource "aws_s3_bucket_object" "fullnode_peers" {
  bucket = aws_s3_bucket.config.id
  key    = "fullnode_peers.config.toml"
  source = "${var.validator_set}/fn/network_peers.config.toml"
  etag   = filemd5("${var.validator_set}/fn/network_peers.config.toml")
}

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
    Name      = "${terraform.workspace}-fullnode-${substr(var.fullnode_ids[count.index], 0, 8)}"
    Role      = "fullnode"
    Workspace = terraform.workspace
    PeerId    = var.fullnode_ids[count.index]
  }

}

data "local_file" "fullnode_keys" {
  count    = var.num_fullnodes
  filename = "${var.validator_set}/fn/${var.fullnode_ids[count.index]}.node.network.keys.toml"
}

resource "aws_secretsmanager_secret" "fullnode_network" {
  count                   = var.num_fullnodes
  name                    = "${terraform.workspace}-fullnode-${substr(var.fullnode_ids[count.index], 0, 8)}"
  recovery_window_in_days = 0
}

resource "aws_secretsmanager_secret_version" "fullnode_network" {
  count         = var.num_fullnodes
  secret_id     = element(aws_secretsmanager_secret.fullnode_network.*.id, count.index)
  secret_string = element(data.local_file.fullnode_keys.*.content, count.index)
}

data "template_file" "fullnode_config" {
  count    = var.num_fullnodes
  template = file("${var.validator_set}/fn/node.config.toml")

  vars = {
    self_ip = element(aws_instance.fullnode.*.private_ip, count.index)
    peer_id = var.fullnode_ids[count.index]
    upstream_peer = var.validator_fullnode_id[local.fullnode_list[count.index]]
  }
}

data "template_file" "fullnode_seeds" {
  count    = var.num_fullnodes
  template = file("templates/seed_peers.config.toml")

  vars = {
    validators = "${var.validator_fullnode_id[local.fullnode_list[count.index]]}:${element(aws_instance.validator.*.private_ip, local.fullnode_list[count.index])}"
    port       = 6181
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
    node_config      = jsonencode(element(data.template_file.fullnode_config.*.rendered, count.index))
    seed_peers       = jsonencode(element(data.template_file.fullnode_seeds.*.rendered, count.index))
    genesis_blob     = jsonencode(filebase64("${var.validator_set}/genesis.blob"))
    peer_id          = var.fullnode_ids[count.index]
    network_secret   = element(aws_secretsmanager_secret.fullnode_network.*.arn, count.index)
    consensus_secret = ""
    fullnode_secret  = ""
    log_level        = var.validator_log_level
    log_group        = var.cloudwatch_logs ? aws_cloudwatch_log_group.testnet.name : ""
    log_region       = var.region
    log_prefix       = "fullnode-${substr(var.fullnode_ids[count.index], 0, 8)}"
    capabilities     = jsonencode(var.validator_linux_capabilities)
    command          = local.validator_command
  }
}

resource "aws_ecs_task_definition" "fullnode" {
  count  = var.num_fullnodes
  family = "${terraform.workspace}-fullnode-${substr(var.fullnode_ids[count.index], 0, 8)}"
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

  volume {
    name      = "consensus-peers"
    host_path = "/opt/libra/consensus_peers.config.toml"
  }

  volume {
    name      = "network-peers"
    host_path = "/opt/libra/network_peers.config.toml"
  }

  volume {
    name      = "fullnode-peers"
    host_path = "/opt/libra/fullnode_peers.config.toml"
  }

  volume {
    name      = "genesis-blob"
    host_path = "/opt/libra/genesis.blob"
  }

  placement_constraints {
    type       = "memberOf"
    expression = "ec2InstanceId == ${element(aws_instance.fullnode.*.id, count.index)}"
  }

  tags = {
    PeerId    = "${substr(var.fullnode_ids[count.index], 0, 8)}"
    Role      = "validator"
    Workspace = terraform.workspace
  }
}

resource "aws_ecs_service" "fullnode" {
  count                              = var.num_fullnodes
  name                               = "${terraform.workspace}-fullnode-${substr(var.fullnode_ids[count.index], 0, 8)}"
  cluster                            = aws_ecs_cluster.testnet.id
  task_definition                    = element(aws_ecs_task_definition.fullnode.*.arn, count.index)
  desired_count                      = 1
  deployment_minimum_healthy_percent = 0

  tags = {
    PeerId    = "${substr(var.fullnode_ids[count.index], 0, 8)}"
    Role      = "fullnode"
    Workspace = terraform.workspace
  }
}
