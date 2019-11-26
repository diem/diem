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

variable "aws_ecs_ami_override" {
  default     = ""
  description = "Machine image to use for ec2 instances"
}

locals {
  aws_ecs_ami = var.aws_ecs_ami_override == "" ? data.aws_ami.ecs.id : var.aws_ecs_ami_override
}

locals {
  ebs_types = ["t2", "t3", "m5", "c5"]

  cpu_by_instance = {
    "t2.small"     = 1024
    "t2.large"     = 2048
    "t2.medium"    = 2048
    "t2.xlarge"    = 4096
    "t3.medium"    = 2048
    "m5.large"     = 2048
    "m5.xlarge"    = 4096
    "m5.2xlarge"   = 8192
    "m5.4xlarge"   = 16384
    "m5.12xlarge"  = 49152
    "m5.24xlarge"  = 98304
    "c5.large"     = 2048
    "c5d.large"    = 2048
    "c5.xlarge"    = 4096
    "c5d.xlarge"   = 4096
    "c5.2xlarge"   = 8192
    "c5d.2xlarge"  = 8192
    "c5.4xlarge"   = 16384
    "c5d.4xlarge"  = 16384
    "c5.9xlarge"   = 36864
    "c5d.9xlarge"  = 36864
    "c5.18xlarge"  = 73728
    "c5d.18xlarge" = 73728
  }

  mem_by_instance = {
    "t2.small"     = 1800
    "t2.medium"    = 3943
    "t2.large"     = 7975
    "t2.xlarge"    = 16039
    "t3.medium"    = 3884
    "m5.large"     = 7680
    "m5.xlarge"    = 15576
    "m5.2xlarge"   = 31368
    "m5.4xlarge"   = 62950
    "m5.12xlarge"  = 189283
    "m5.24xlarge"  = 378652
    "c5.large"     = 3704
    "c5d.large"    = 3704
    "c5.xlarge"    = 7624
    "c5d.xlarge"   = 7624
    "c5.2xlarge"   = 15463
    "c5d.2xlarge"  = 15463
    "c5.4xlarge"   = 31142
    "c5d.4xlarge"  = 31142
    "c5.9xlarge"   = 70341
    "c5d.9xlarge"  = 70341
    "c5.18xlarge"  = 140768
    "c5d.18xlarge" = 140768
  }
}

resource "aws_cloudwatch_log_group" "testnet" {
  name              = terraform.workspace
  retention_in_days = 7
}

resource "aws_cloudwatch_log_metric_filter" "log_metric_filter" {
  count          = var.cloudwatch_logs ? 1 : 0
  name           = "critical_log"
  pattern        = "[code=C*, time, x, file, ...]"
  log_group_name = aws_cloudwatch_log_group.testnet.name

  metric_transformation {
    name      = "critical_lines"
    namespace = "LogMetrics"
    value     = "1"
  }
}

resource "random_id" "bucket" {
  byte_length = 8
}

resource "aws_s3_bucket" "config" {
  bucket = "libra-${terraform.workspace}-${random_id.bucket.hex}"
  region = var.region
}

resource "aws_s3_bucket_public_access_block" "config" {
  bucket                  = aws_s3_bucket.config.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_object" "network_peers" {
  bucket = aws_s3_bucket.config.id
  key    = "network_peers.config.toml"
  source = "${var.validator_set}/val/network_peers.config.toml"
  etag   = filemd5("${var.validator_set}/val/network_peers.config.toml")
}

resource "aws_s3_bucket_object" "consensus_peers" {
  bucket = aws_s3_bucket.config.id
  key    = "consensus_peers.config.toml"
  source = "${var.validator_set}/consensus_peers.config.toml"
  etag   = filemd5("${var.validator_set}/consensus_peers.config.toml")
}

resource "aws_s3_bucket_object" "genesis_blob" {
  bucket = aws_s3_bucket.config.id
  key    = "genesis.blob"
  source = "${var.validator_set}/genesis.blob"
  etag   = filemd5("${var.validator_set}/genesis.blob")
}

data "template_file" "user_data" {
  template = file("templates/ec2_user_data.sh")

  vars = {
    ecs_cluster     = aws_ecs_cluster.testnet.name
    network_peers   = "s3://${aws_s3_bucket.config.id}/${aws_s3_bucket_object.network_peers.id}"
    fullnode_peers  = "s3://${aws_s3_bucket.config.id}/${aws_s3_bucket_object.fullnode_peers.id}"
    consensus_peers = "s3://${aws_s3_bucket.config.id}/${aws_s3_bucket_object.consensus_peers.id}"
    genesis_blob    = "s3://${aws_s3_bucket.config.id}/${aws_s3_bucket_object.genesis_blob.id}"
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
    Name      = "${terraform.workspace}-validator-${substr(var.peer_ids[count.index], 0, 8)}"
    Role      = "validator"
    Workspace = terraform.workspace
    PeerId    = var.peer_ids[count.index]
  }

}

data "local_file" "consensus_keys" {
  count    = length(var.peer_ids)
  filename = "${var.validator_set}/val/${var.peer_ids[count.index]}.node.consensus.keys.toml"
}

resource "aws_secretsmanager_secret" "validator_consensus" {
  count                   = length(var.peer_ids)
  name                    = "${terraform.workspace}-consensus-${substr(var.peer_ids[count.index], 0, 8)}"
  recovery_window_in_days = 0
}

resource "aws_secretsmanager_secret_version" "validator_consensus" {
  count         = length(var.peer_ids)
  secret_id     = element(aws_secretsmanager_secret.validator_consensus.*.id, count.index)
  secret_string = element(data.local_file.consensus_keys.*.content, count.index)
}

data "local_file" "network_keys" {
  count    = length(var.peer_ids)
  filename = "${var.validator_set}/val/${var.peer_ids[count.index]}.node.network.keys.toml"
}

resource "aws_secretsmanager_secret" "validator_network" {
  count                   = length(var.peer_ids)
  name                    = "${terraform.workspace}-network-${substr(var.peer_ids[count.index], 0, 8)}"
  recovery_window_in_days = 0
}

resource "aws_secretsmanager_secret_version" "validator_network" {
  count         = length(var.peer_ids)
  secret_id     = element(aws_secretsmanager_secret.validator_network.*.id, count.index)
  secret_string = element(data.local_file.network_keys.*.content, count.index)
}

data "local_file" "validator_fullnode_keys" {
  count    = length(var.peer_ids)
  filename = "${var.validator_set}/val/${var.validator_fullnode_id[0]}.network.keys.toml"
}

resource "aws_secretsmanager_secret" "validator_fullnode" {
  count                   = 1
  name                    = "${terraform.workspace}-validator_fullnode-${substr(var.validator_fullnode_id[0], 0, 8)}"
  recovery_window_in_days = 0
}

resource "aws_secretsmanager_secret_version" "validator_fullnode" {
  count         = length(var.peer_ids)
  secret_id     = element(aws_secretsmanager_secret.validator_fullnode.*.id, count.index)
  secret_string = element(data.local_file.validator_fullnode_keys.*.content, count.index)
}

data "template_file" "validator_config" {
  count    = length(var.peer_ids)
  template = file("${var.validator_set}/val/node.config.toml")

  vars = {
    self_ip     = var.validator_use_public_ip == true ? element(aws_instance.validator.*.public_ip, count.index) : element(aws_instance.validator.*.private_ip, count.index)
    peer_id     = var.peer_ids[count.index]
    fullnode_id = var.validator_fullnode_id[0]
  }
}

data "template_file" "seed_peers" {
  template = file("templates/seed_peers.config.toml")

  vars = {
    validators = join(",", formatlist("%s:%s", slice(var.peer_ids, 0, 3), var.validator_use_public_ip == true ? slice(aws_instance.validator.*.public_ip, 0, 3) : slice(aws_instance.validator.*.private_ip, 0, 3)))
    port       = 6180
  }
}

variable "log_to_file" {
  type        = bool
  default     = false
  description = "Set to true to log to /opt/libra/data/libra.log (in container) and /data/libra/libra.log (on host). This file won't be log rotated, you need to handle log rotation on your own if you choose this option"
}

locals {
  validator_command = var.log_to_file ? jsonencode(["bash", "-c", "/docker-run.sh >> /opt/libra/data/libra.log 2>&1"]) : ""
}

data "template_file" "ecs_task_definition" {
  count    = length(var.peer_ids)
  template = file("templates/validator.json")

  vars = {
    image            = local.image_repo
    image_version    = local.image_version
    cpu              = local.cpu_by_instance[var.validator_type]
    mem              = local.mem_by_instance[var.validator_type]
    node_config      = jsonencode(element(data.template_file.validator_config.*.rendered, count.index))
    seed_peers       = jsonencode(data.template_file.seed_peers.rendered)
    genesis_blob     = jsonencode(filebase64("${var.validator_set}/genesis.blob"))
    peer_id          = var.peer_ids[count.index]
    network_secret   = element(aws_secretsmanager_secret.validator_network.*.arn, count.index)
    consensus_secret = element(aws_secretsmanager_secret.validator_consensus.*.arn, count.index)
    fullnode_secret  = element(aws_secretsmanager_secret.validator_fullnode.*.arn, count.index)
    log_level        = var.validator_log_level
    log_group        = var.cloudwatch_logs ? aws_cloudwatch_log_group.testnet.name : ""
    log_region       = var.region
    log_prefix       = "validator-${substr(var.peer_ids[count.index], 0, 8)}"
    capabilities     = jsonencode(var.validator_linux_capabilities)
    command          = local.validator_command
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
