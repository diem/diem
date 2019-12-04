locals {
  faucet_image_repo = var.faucet_image_repo
  faucet_image_tag  = var.faucet_image_tag
}

resource "aws_instance" "faucet" {
  ami                         = local.aws_ecs_ami
  instance_type               = "t3.medium"
  subnet_id                   = element(aws_subnet.testnet.*.id, 0)
  depends_on                  = [aws_main_route_table_association.testnet]
  vpc_security_group_ids      = [aws_security_group.faucet-host.id]
  associate_public_ip_address = local.instance_public_ip
  key_name                    = aws_key_pair.libra.key_name
  iam_instance_profile        = aws_iam_instance_profile.ecsInstanceRole.name
  user_data                   = local.user_data

  tags = {
    Name      = "${terraform.workspace}-faucet"
    Role      = "faucet"
    Workspace = terraform.workspace
  }
}

data "template_file" "ecs_faucet_definition" {
  template = file("templates/faucet.json")

  vars = {
    faucet_image_repo    = local.faucet_image_repo
    faucet_image_tag_str = substr(var.image_tag, 0, 6) == "sha256" ? "@${local.faucet_image_tag}" : ":${local.faucet_image_tag}"
    ac_hosts             = join(",", aws_instance.validator.*.private_ip)
    cfg_num_validators   = var.num_validators
    cfg_seed             = var.config_seed
    log_level            = var.faucet_log_level
    log_group            = var.cloudwatch_logs ? aws_cloudwatch_log_group.testnet.name : ""
    log_region           = var.region
    log_prefix           = "faucet"
  }
}

resource "aws_ecs_task_definition" "faucet" {
  family                = "${terraform.workspace}-faucet"
  container_definitions = data.template_file.ecs_faucet_definition.rendered
  execution_role_arn    = aws_iam_role.ecsTaskExecutionRole.arn

  placement_constraints {
    type       = "memberOf"
    expression = "ec2InstanceId == ${aws_instance.faucet.id}"
  }

  tags = {
    Role      = "faucet"
    Workspace = terraform.workspace
  }
}

resource "aws_ecs_service" "faucet" {
  name                               = "${terraform.workspace}-faucet"
  cluster                            = aws_ecs_cluster.testnet.id
  task_definition                    = aws_ecs_task_definition.faucet.arn
  desired_count                      = 1
  deployment_minimum_healthy_percent = 0
  count                              = var.cluster_test ? 0 : 1

  tags = {
    Role      = "faucet"
    Workspace = terraform.workspace
  }
}
