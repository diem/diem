variable "cluster_test" {
  default     = false
  description = "Setup additional resources for cluster test"
}

resource "aws_iam_role" "cluster-test-runner" {
  name               = "ClusterTestRunner-Role-${var.region}-${terraform.workspace}"
  assume_role_policy = data.aws_iam_policy_document.instance-assume-role.json
}

data "aws_caller_identity" "current" {}

data "aws_iam_policy_document" "cluster-test-runner" {
  statement {
    actions   = ["ec2:Describe*", "ecr:Describe*", "ecr:BatchGet*"]
    resources = ["*"]
  }
  statement {
    actions   = ["ecs:*"]
    resources = ["arn:aws:ecs:${var.region}:${data.aws_caller_identity.current.account_id}:service/${terraform.workspace}/*"]
  }
}

resource "aws_iam_role_policy" "cluster-test-runner" {
  name   = "ClusterTestRunner-Policy-${var.region}-${terraform.workspace}"
  role   = "${aws_iam_role.cluster-test-runner.name}"
  policy = data.aws_iam_policy_document.cluster-test-runner.json
}

resource "aws_iam_instance_profile" "cluster-test-runner" {
  name = "ClusterTestRunner-IamProfile-${var.region}-${terraform.workspace}"
  role = "${aws_iam_role.cluster-test-runner.name}"
}

data "template_file" "cluster_test_user_data" {
  template = file("templates/cluster_test_user_data.sh")

  vars = {
    ssh_key = file(var.ssh_priv_key_file)
    ct      = file("templates/cluster-test/ct")
  }
}

resource "aws_instance" "cluster-test-runner" {
  ami                         = local.aws_ecs_ami
  instance_type               = "c5.2xlarge"
  subnet_id                   = element(aws_subnet.testnet.*.id, 0)
  vpc_security_group_ids      = [aws_security_group.cluster-test-host.id]
  associate_public_ip_address = local.instance_public_ip
  key_name                    = aws_key_pair.libra.key_name
  user_data                   = data.template_file.cluster_test_user_data.rendered
  count                       = var.cluster_test ? 1 : 0

  tags = {
    Name      = "${terraform.workspace}-cluster-test-runner"
    Role      = "cluster-test-runner"
    Workspace = terraform.workspace
  }
  iam_instance_profile = "${aws_iam_instance_profile.cluster-test-runner.name}"

  root_block_device {
      volume_type = "gp2"
      volume_size = 1024
  }
}

resource "aws_security_group" "cluster-test-host" {
  name        = "${terraform.workspace}-cluster-test-host"
  description = "Benchmarker host"
  vpc_id      = aws_vpc.testnet.id
}

resource "aws_security_group_rule" "validator-svc-debug" {
  security_group_id        = aws_security_group.validator.id
  type                     = "ingress"
  from_port                = 30306
  to_port                  = 30306
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.cluster-test-host.id
}

resource "aws_security_group_rule" "prometheus-query" {
  security_group_id        = aws_security_group.monitoring.id
  type                     = "ingress"
  from_port                = 9091
  to_port                  = 9091
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.cluster-test-host.id
}

resource "aws_security_group_rule" "cluster-test-svc-mon" {
  security_group_id        = aws_security_group.cluster-test-host.id
  type                     = "ingress"
  from_port                = 14297
  to_port                  = 14297
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.monitoring.id
}

resource "aws_security_group_rule" "cluster-test-host-mon" {
  security_group_id        = aws_security_group.cluster-test-host.id
  type                     = "ingress"
  from_port                = 9100
  to_port                  = 9100
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.monitoring.id
}

resource "aws_security_group_rule" "cluster-test-ssh-validator" {
  security_group_id        = aws_security_group.validator.id
  type                     = "ingress"
  from_port                = 22
  to_port                  = 22
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.cluster-test-host.id
}

resource "aws_security_group_rule" "cluster-test-debug-port-validator" {
  security_group_id        = aws_security_group.validator.id
  type                     = "ingress"
  from_port                = 6191
  to_port                  = 6191
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.cluster-test-host.id
}

resource "aws_security_group_rule" "cluster-test-metrics-port-validator" {
  security_group_id        = aws_security_group.validator.id
  type                     = "ingress"
  from_port                = 9101
  to_port                  = 9101
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.cluster-test-host.id
}

resource "aws_security_group_rule" "cluster-test-egress" {
  security_group_id = aws_security_group.cluster-test-host.id
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  ipv6_cidr_blocks  = ["::/0"]
}
