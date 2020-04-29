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
  name                 = "${terraform.workspace}-ecsInstanceRole"
  path                 = var.iam_path
  assume_role_policy   = data.aws_iam_policy_document.instance-assume-role.json
  permissions_boundary = var.permissions_boundary_policy

  tags = {
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
}

resource "aws_iam_role_policy_attachment" "ecsInstanceRole_" {
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
  statement {
    actions   = ["kms:Encrypt", "kms:Decrypt", "kms:DescribeKey"]
    resources = [aws_kms_key.vault.arn]
  }
}

resource "aws_iam_role_policy" "ecs_extra" {
  name   = "Config-S3"
  role   = aws_iam_role.ecsInstanceRole.name
  policy = data.aws_iam_policy_document.ecs_extra.json
}

resource "aws_iam_instance_profile" "ecsInstanceRole" {
  name = "${terraform.workspace}-ecsInstanceRole"
  path = var.iam_path
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
  name                 = "${terraform.workspace}-ecsTaskExecutionRole"
  path                 = var.iam_path
  assume_role_policy   = data.aws_iam_policy_document.task-assume-role.json
  permissions_boundary = var.permissions_boundary_policy

  tags = {
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
}

resource "aws_iam_role_policy_attachment" "ecsTaskExecutionRole_" {
  role       = aws_iam_role.ecsTaskExecutionRole.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_key_pair" "libra" {
  key_name   = "${terraform.workspace}-libra"
  public_key = var.ssh_pub_key
}
