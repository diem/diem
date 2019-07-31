data "aws_ami" "ecs" {
  most_recent = true

  filter {
    name   = "name"
    values = ["amzn2-ami-ecs-hvm-2.0.*"]
  }

  filter {
    name   = "architecture"
    values = ["x86_64"]
  }

  owners = ["amazon"]
}

locals {
  cpu_by_instance = {
    "t2.large"    = 2048
    "t2.medium"   = 2048
    "t3.medium"   = 2048
    "m5.large"    = 2048
    "m5.xlarge"   = 4096
    "m5.2xlarge"  = 8192
    "m5.4xlarge"  = 16384
    "m5.12xlarge" = 49152
    "m5.24xlarge" = 98304
    "c5.large"    = 2048
    "c5.xlarge"   = 4096
    "c5.2xlarge"  = 8192
    "c5.4xlarge"  = 16384
    "c5.9xlarge"  = 36864
    "c5.18xlarge" = 73728
  }

  mem_by_instance = {
    "t2.medium"   = 3943
    "t2.large"    = 7975
    "t3.medium"   = 3884
    "m5.large"    = 7680
    "m5.xlarge"   = 15576
    "m5.2xlarge"  = 31368
    "m5.4xlarge"  = 62950
    "m5.12xlarge" = 189283
    "m5.24xlarge" = 378652
    "c5.large"    = 3704
    "c5.xlarge"   = 7624
    "c5.2xlarge"  = 15464
    "c5.4xlarge"  = 31142
    "c5.9xlarge"  = 70341
    "c5.18xlarge" = 140768
  }
}

resource "aws_cloudwatch_log_group" "testnet" {
  name              = terraform.workspace
  retention_in_days = 7
}

resource "aws_cloudwatch_log_metric_filter" "log_metric_filter" {
  name           = "critical_log"
  pattern        = "[code=C*, time, x, file, ...]"
  log_group_name = "${aws_cloudwatch_log_group.testnet.name}"

  metric_transformation {
    name      = "critical_lines"
    namespace = "LogMetrics"
    value     = "1"
  }
}

data "template_file" "user_data" {
  template = file("templates/ec2_user_data.sh")

  vars = {
    ecs_cluster = aws_ecs_cluster.testnet.name
  }
}

locals {
  image_repo         = var.image_repo
  instance_public_ip = true
  user_data          = data.template_file.user_data.rendered
  image_version      = substr(var.image_tag, 0, 6) == "sha256" ? "@${var.image_tag}" : ":${var.image_tag}"
}

resource "aws_instance" "validator" {
  count         = length(var.peer_ids)
  ami           = data.aws_ami.ecs.id
  instance_type = var.validator_type
  subnet_id = element(
    aws_subnet.testnet.*.id,
    count.index % length(data.aws_availability_zones.available.names),
  )
  vpc_security_group_ids      = [aws_security_group.validator.id]
  associate_public_ip_address = local.instance_public_ip
  key_name                    = aws_key_pair.libra.key_name
  iam_instance_profile        = aws_iam_instance_profile.ecsInstanceRole.name
  user_data                   = local.user_data

  root_block_device {
    volume_type = "io1"
    volume_size = var.validator_ebs_size
    iops        = var.validator_ebs_size * 50 # max 50iops/gb
  }

  tags = {
    Name      = "${terraform.workspace}-validator-${substr(var.peer_ids[count.index], 0, 8)}"
    Role      = "validator"
    Workspace = terraform.workspace
    PeerId    = var.peer_ids[count.index]
  }
}

data "local_file" "keys" {
  count    = length(var.peer_ids)
  filename = "${var.validator_set}/${var.peer_ids[count.index]}.node.keys.toml"
}

resource "aws_secretsmanager_secret" "validator" {
  count                   = length(var.peer_ids)
  name                    = "${terraform.workspace}-${substr(var.peer_ids[count.index], 0, 8)}"
  recovery_window_in_days = 0
}

resource "aws_secretsmanager_secret_version" "validator" {
  count         = length(var.peer_ids)
  secret_id     = element(aws_secretsmanager_secret.validator.*.id, count.index)
  secret_string = element(data.local_file.keys.*.content, count.index)
}

data "template_file" "node_config" {
  count = length(var.peer_ids)
  template = file("${var.validator_set}/node.config.toml")

  vars = {
    self_ip = element(aws_instance.validator.*.private_ip, count.index)
  }
}

data "template_file" "seed_peers" {
  template = file("templates/seed_peers.config.toml")

  vars = {
    validators = join(",", formatlist("%s:%s", slice(var.peer_ids, 0, 3), slice(aws_instance.validator.*.private_ip, 0, 3)))
  }
}

data "template_file" "ecs_task_definition" {
  count    = length(var.peer_ids)
  template = file("templates/validator.json")

  vars = {
    image         = local.image_repo
    image_version = local.image_version
    cpu           = local.cpu_by_instance[var.validator_type]
    mem           = local.mem_by_instance[var.validator_type]
    node_config   = jsonencode(element(data.template_file.node_config.*.rendered, count.index))
    seed_peers    = jsonencode(data.template_file.seed_peers.rendered)
    trusted_peers = jsonencode(file("${var.validator_set}/trusted_peers.config.toml"))
    genesis_blob  = jsonencode(filebase64("${var.validator_set}/genesis.blob"))
    peer_id       = var.peer_ids[count.index]
    secret        = element(aws_secretsmanager_secret.validator.*.arn, count.index)
    log_level     = var.validator_log_level
    log_group     = aws_cloudwatch_log_group.testnet.name
    log_region    = var.region
    log_prefix    = "validator-${substr(var.peer_ids[count.index], 0, 8)}"
  }
}

resource "aws_ecs_task_definition" "validator" {
  count  = length(var.peer_ids)
  family = "${terraform.workspace}-validator-${substr(var.peer_ids[count.index], 0, 8)}"
  container_definitions = element(
    data.template_file.ecs_task_definition.*.rendered,
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
    expression = "ec2InstanceId == ${element(aws_instance.validator.*.id, count.index)}"
  }

  tags = {
    PeerId    = "${substr(var.peer_ids[count.index], 0, 8)}"
    Role      = "validator"
    Workspace = terraform.workspace
  }
}

resource "aws_ecs_cluster" "testnet" {
  name = terraform.workspace
}

resource "aws_ecs_service" "validator" {
  count                              = length(var.peer_ids)
  name                               = "${terraform.workspace}-validator-${substr(var.peer_ids[count.index], 0, 8)}"
  cluster                            = aws_ecs_cluster.testnet.id
  task_definition                    = element(aws_ecs_task_definition.validator.*.arn, count.index)
  desired_count                      = 1
  deployment_minimum_healthy_percent = 0

  tags = {
    PeerId    = "${substr(var.peer_ids[count.index], 0, 8)}"
    Role      = "validator"
    Workspace = terraform.workspace
  }
}
