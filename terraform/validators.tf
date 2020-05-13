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

  tags = {
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
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

  tags = {
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
}

resource "aws_s3_bucket_public_access_block" "config" {
  bucket                  = aws_s3_bucket.config.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}


data "template_file" "user_data" {
  template = file("templates/ec2_user_data.sh")

  vars = {
    persistent          = var.persist_libra_data
    ecs_cluster         = aws_ecs_cluster.testnet.name
    host_log_path       = "libra.log"
    host_structlog_path = "libra_structlog.log"
    enable_logrotate    = var.log_to_file || var.enable_logstash
  }
}

locals {
  image_repo                 = var.image_repo
  image_version              = substr(var.image_tag, 0, 6) == "sha256" ? "@${var.image_tag}" : ":${var.image_tag}"
  override_image_versions    = [for img in var.override_image_tags : substr(img, 0, 6) == "sha256" ? "@${img}" : ":${img}"]
  safety_rules_image_repo    = var.safety_rules_image_repo
  safety_rules_image_version = substr(var.safety_rules_image_tag, 0, 6) == "sha256" ? "@${var.safety_rules_image_tag}" : ":${var.safety_rules_image_tag}"
  instance_public_ip         = true
  user_data                  = data.template_file.user_data.rendered
}

resource "aws_ebs_snapshot" "restore_snapshot" {
  count     = length(var.restore_vol_ids) == 0 ? 0 : var.num_validators
  volume_id = length(var.restore_vol_ids) == 0 ? "" : var.restore_vol_ids[count.index]

  tags = {
    Name      = "${terraform.workspace}-validator-${count.index}-restore-snap"
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
}

resource "aws_instance" "validator" {
  count         = var.num_validators
  ami           = local.aws_ecs_ami
  instance_type = var.validator_type
  subnet_id = element(
    aws_subnet.testnet.*.id,
    count.index % length(data.aws_availability_zones.available.names),
  )
  depends_on                  = [aws_main_route_table_association.testnet, aws_iam_role_policy.ecs_extra]
  vpc_security_group_ids      = [aws_security_group.validator.id]
  private_ip                  = length(var.override_validator_ips) == 0 ? null : var.override_validator_ips[count.index]
  associate_public_ip_address = local.instance_public_ip
  key_name                    = aws_key_pair.libra.key_name
  iam_instance_profile        = aws_iam_instance_profile.ecsInstanceRole.name
  user_data                   = local.user_data

  dynamic "ebs_block_device" {
    for_each = contains(local.ebs_types, split(".", var.validator_type)[0]) ? [0] : []
    content {
      device_name = "/dev/xvdb"
      volume_type = "io1"
      volume_size = length(var.restore_vol_ids) == 0 ? var.validator_ebs_size : aws_ebs_snapshot.restore_snapshot[count.index].volume_size
      snapshot_id = length(var.restore_vol_ids) == 0 ? "" : aws_ebs_snapshot.restore_snapshot[count.index].id
      iops        = var.validator_ebs_size * 50 # max 50iops/gb
    }
  }

  tags = {
    Name      = "${terraform.workspace}-validator-${count.index}"
    Role      = "validator"
    Terraform = "testnet"
    Workspace = terraform.workspace
    NodeIndex = count.index
  }
}

locals {
  seed_peer_ip           = aws_instance.validator.0.private_ip
  validator_command      = var.log_to_file || var.enable_logstash ? jsonencode(["bash", "-c", "/docker-run-dynamic.sh >> ${var.log_path} 2>&1"]) : ""
  aws_elasticsearch_host = var.enable_logstash ? join(",", aws_elasticsearch_domain.logging.*.endpoint) : ""
  logstash_config        = "input { file { path => '${var.structlog_path}'\\n codec => 'json'\\n}}\\n filter {  json {  \\nsource => 'message'\\n}}\\n output {  amazon_es { \\nhosts => ['https://${local.aws_elasticsearch_host}']\\nregion => 'us-west-2'\\nindex => 'validator-logs-%%{+YYYY.MM.dd}'\\n}}"
}

data "template_file" "ecs_task_definition" {
  count    = var.num_validators
  template = file("templates/validator.json")

  vars = {
    image                         = local.image_repo
    image_version                 = local.override_image_versions == [] ? local.image_version : local.override_image_versions[count.index % length(local.override_image_versions)]
    cpu                           = (var.enable_logstash ? local.cpu_by_instance[var.validator_type] - 584 : local.cpu_by_instance[var.validator_type]) - 512
    mem                           = (var.enable_logstash ? local.mem_by_instance[var.validator_type] - 1024 : local.mem_by_instance[var.validator_type]) - 256
    cfg_listen_addr               = var.validator_use_public_ip == true ? element(aws_instance.validator.*.public_ip, count.index) : element(aws_instance.validator.*.private_ip, count.index)
    cfg_node_index                = count.index
    cfg_num_validators            = var.cfg_num_validators_override == 0 ? var.num_validators : var.cfg_num_validators_override
    cfg_num_validators_in_genesis = var.num_validators_in_genesis == 0 ? var.num_validators : var.num_validators_in_genesis
    cfg_seed                      = var.config_seed
    cfg_seed_peer_ip              = local.seed_peer_ip

    cfg_fullnode_seed = count.index < var.num_fullnode_networks ? var.fullnode_seed : ""
    cfg_num_fullnodes = var.num_fullnodes

    log_level                  = var.validator_log_level
    log_group                  = var.cloudwatch_logs ? aws_cloudwatch_log_group.testnet.name : ""
    log_region                 = var.region
    log_prefix                 = "validator-${count.index}"
    capabilities               = jsonencode(var.validator_linux_capabilities)
    command                    = local.validator_command
    logstash                   = var.enable_logstash
    logstash_image             = var.logstash_image
    logstash_version           = ":${var.logstash_version}"
    logstash_config            = local.logstash_config
    safety_rules_image         = local.safety_rules_image_repo
    safety_rules_image_version = local.safety_rules_image_version
    push_metrics_endpoint      = "http://${aws_instance.monitoring.private_ip}:9092/metrics/job/safety_rules/role/validator/peer_id/val-${count.index}"
    cfg_vault_addr             = "http://${aws_instance.vault.private_ip}:8200"
    cfg_vault_namespace        = "val-${count.index}"
    use_vault                  = var.safety_rules_use_vault
    structlog_path             = var.log_to_file || var.enable_logstash ? var.structlog_path : ""
  }
}

resource "aws_ecs_task_definition" "validator" {
  count  = var.num_validators
  family = "${terraform.workspace}-validator-${count.index}"
  container_definitions = element(
    data.template_file.ecs_task_definition.*.rendered,
    count.index
  )
  execution_role_arn = aws_iam_role.ecsTaskExecutionRole.arn
  network_mode       = "host"

  volume {
    name      = "libra-data"
    host_path = var.persist_libra_data ? "/data/libra" : ""
  }

  placement_constraints {
    type       = "memberOf"
    expression = "ec2InstanceId == ${element(aws_instance.validator.*.id, count.index)}"
  }

  tags = {
    NodeIndex = count.index
    Role      = "validator"
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
}

resource "aws_ecs_cluster" "testnet" {
  name = terraform.workspace

  tags = {
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
}

resource "aws_ecs_service" "validator" {
  count                              = var.num_validators
  name                               = "${terraform.workspace}-validator-${count.index}"
  cluster                            = aws_ecs_cluster.testnet.id
  task_definition                    = element(aws_ecs_task_definition.validator.*.arn, count.index)
  desired_count                      = 1
  deployment_minimum_healthy_percent = 0

  tags = {
    NodeIndex = count.index
    Role      = "validator"
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
}
