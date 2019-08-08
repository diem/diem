resource "aws_iam_role" "ecsInstanceRole" {
  name = "${terraform.workspace}-ecsInstanceRole"

  assume_role_policy = <<EOF
{
  "Version": "2008-10-17",
  "Statement": [
    {
      "Sid": "",
      "Effect": "Allow",
      "Principal": {
        "Service": "ec2.amazonaws.com"
      },
      "Action": "sts:AssumeRole"
    }
  ]
}
EOF

}

resource "aws_iam_role_policy_attachment" "ecsInstanceRole" {
  role       = aws_iam_role.ecsInstanceRole.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonEC2ContainerServiceforEC2Role"
}

data "aws_iam_policy_document" "ecs_extra" {
  statement {
    actions   = ["s3:GetObject"]
    resources = ["arn:aws:s3:::${aws_s3_bucket.config.id}/${aws_s3_bucket_object.trusted_peers.id}"]
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

resource "aws_iam_role" "ecsTaskExecutionRole" {
  name = "${terraform.workspace}-ecsTaskExecutionRole"

  assume_role_policy = <<EOF
{
  "Version": "2008-10-17",
  "Statement": [
    {
      "Sid": "",
      "Effect": "Allow",
      "Principal": {
        "Service": "ecs-tasks.amazonaws.com"
      },
      "Action": "sts:AssumeRole"
    }
  ]
}
EOF

}

resource "aws_iam_role_policy_attachment" "ecsTaskExecutionRole" {
  role       = aws_iam_role.ecsTaskExecutionRole.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

locals {
  secrets_arn = split(":", aws_secretsmanager_secret.validator[0].arn)
}

resource "aws_iam_policy" "validator" {
  name = "${terraform.workspace}-validator"

  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue"
      ],
      "Resource": [
        "${join(
  ":",
  slice(local.secrets_arn, 0, length(local.secrets_arn) - 1),
)}:*"
      ]
    }
  ]
}
EOF

}

resource "aws_iam_role_policy_attachment" "ecsTaskExecutionRole-secrets" {
  role       = aws_iam_role.ecsTaskExecutionRole.name
  policy_arn = aws_iam_policy.validator.arn
}

resource "aws_key_pair" "libra" {
  key_name   = "${terraform.workspace}-libra"
  public_key = var.ssh_pub_key
}

