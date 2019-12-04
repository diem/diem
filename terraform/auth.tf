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

resource "aws_key_pair" "libra" {
  key_name   = "${terraform.workspace}-libra"
  public_key = var.ssh_pub_key
}
