data "aws_iam_policy_document" "instance-assume-role" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "ecsInstanceRole" {
  name               = "${terraform.workspace}-ecsInstanceRole"
  assume_role_policy = data.aws_iam_policy_document.instance-assume-role.json
}

resource "aws_iam_role_policy_attachment" "ecsInstanceRole" {
  role       = aws_iam_role.ecsInstanceRole.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonEC2ContainerServiceforEC2Role"
}

data "aws_iam_policy_document" "ecs_extra" {
  statement {
    actions = ["s3:GetObject"]
    resources = [
      "arn:aws:s3:::${aws_s3_bucket.config.id}/*",
    ]
  }
}

resource "aws_iam_policy" "ecs_extra" {
  name   = "${terraform.workspace}-ECS-extra"
  policy = data.aws_iam_policy_document.ecs_extra.json
}

resource "aws_iam_role_policy_attachment" "ecs_extra" {
  role       = aws_iam_role.ecsInstanceRole.name
  policy_arn = aws_iam_policy.ecs_extra.arn
}

resource "aws_iam_instance_profile" "ecsInstanceRole" {
  name = "${terraform.workspace}-ecsInstanceRole"
  role = aws_iam_role.ecsInstanceRole.name
}

data "aws_iam_policy_document" "task-assume-role" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "ecsTaskExecutionRole" {
  name               = "${terraform.workspace}-ecsTaskExecutionRole"
  assume_role_policy = data.aws_iam_policy_document.task-assume-role.json
}

resource "aws_iam_role_policy_attachment" "ecsTaskExecutionRole" {
  role       = aws_iam_role.ecsTaskExecutionRole.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

locals {
  consensus_secrets_arn = split(":", aws_secretsmanager_secret.validator_network[0].arn)
  network_secrets_arn   = split(":", aws_secretsmanager_secret.validator_consensus[0].arn)
}

data "aws_iam_policy_document" "validator" {
  statement {
    actions = ["secretsmanager:GetSecretValue"]
    resources = [
      "${join(":", slice(local.consensus_secrets_arn, 0, length(local.consensus_secrets_arn) - 1), )}:*",
      "${join(":", slice(local.network_secrets_arn, 0, length(local.network_secrets_arn) - 1), )}:*"
    ]
  }
}

resource "aws_iam_policy" "validator" {
  name   = "${terraform.workspace}-validator"
  policy = data.aws_iam_policy_document.validator.json
}

resource "aws_iam_role_policy_attachment" "ecsTaskExecutionRole-secrets" {
  role       = aws_iam_role.ecsTaskExecutionRole.name
  policy_arn = aws_iam_policy.validator.arn
}

resource "aws_key_pair" "libra" {
  key_name   = "${terraform.workspace}-libra"
  public_key = var.ssh_pub_key
}
