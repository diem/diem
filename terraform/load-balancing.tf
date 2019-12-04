locals {
  zone_id    = var.zone_id
  dns_suffix = var.append_workspace_dns ? ".${terraform.workspace}" : ""
}

resource "aws_route53_record" "monitoring" {
  count   = local.zone_id == "" ? 0 : 1
  zone_id = local.zone_id
  name    = "prometheus${local.dns_suffix}"
  type    = "A"
  ttl     = 60
  records = [aws_instance.monitoring.public_ip]
}

resource "aws_lb" "validator-ac" {
  name                             = "${terraform.workspace}-ac"
  load_balancer_type               = "network"
  enable_cross_zone_load_balancing = true
  subnets                          = aws_subnet.testnet.*.id
}

resource "aws_lb_target_group" "validator-ac" {
  name     = "${terraform.workspace}-ac"
  protocol = "TCP"
  port     = 8000
  vpc_id   = aws_vpc.testnet.id
}

resource "aws_lb_target_group_attachment" "validator-ac" {
  count            = var.cluster_test ? 0 : var.num_validators
  target_group_arn = aws_lb_target_group.validator-ac.arn
  target_id        = element(aws_instance.validator.*.id, count.index)
}

resource "aws_lb_listener" "validator-ac" {
  load_balancer_arn = aws_lb.validator-ac.arn
  port              = 8000
  protocol          = "TCP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.validator-ac.arn
  }
}

resource "aws_lb_target_group" "fullnode-ac" {
  name     = "${terraform.workspace}-fullnode"
  protocol = "TCP"
  port     = 8000
  vpc_id   = aws_vpc.testnet.id
}

resource "aws_lb_target_group_attachment" "fullnode-ac" {
  count            = var.num_fullnodes
  target_group_arn = aws_lb_target_group.fullnode-ac.arn
  target_id        = element(aws_instance.fullnode.*.id, count.index)
}

resource "aws_lb_listener" "fullnode-ac" {
  load_balancer_arn = aws_lb.validator-ac.arn
  port              = 8001
  protocol          = "TCP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.fullnode-ac.arn
  }
}

resource "aws_route53_record" "validator-ac" {
  count   = local.zone_id == "" ? 0 : 1
  zone_id = local.zone_id
  name    = "ac${local.dns_suffix}"
  type    = "A"

  alias {
    name                   = aws_lb.validator-ac.dns_name
    zone_id                = aws_lb.validator-ac.zone_id
    evaluate_target_health = true
  }
}

# FAUCET #

resource "aws_lb" "faucet" {
  name                             = "${terraform.workspace}-faucet"
  load_balancer_type               = "application"
  enable_cross_zone_load_balancing = true
  subnets                          = aws_subnet.testnet.*.id
  security_groups                  = [aws_security_group.faucet-lb.id]
}

resource "aws_lb_target_group" "faucet" {
  name     = "${terraform.workspace}-faucet"
  protocol = "HTTP"
  port     = 8000
  vpc_id   = aws_vpc.testnet.id
}

resource "aws_lb_target_group_attachment" "faucet" {
  count            = 1
  target_group_arn = aws_lb_target_group.faucet.arn
  target_id        = aws_instance.faucet.id
}

resource "aws_lb_listener" "faucet" {
  load_balancer_arn = aws_lb.faucet.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.faucet.arn
  }
}

resource "aws_route53_record" "faucet" {
  count   = local.zone_id == "" ? 0 : 1
  zone_id = local.zone_id
  name    = "faucet${local.dns_suffix}"
  type    = "A"

  alias {
    name                   = aws_lb.faucet.dns_name
    zone_id                = aws_lb.faucet.zone_id
    evaluate_target_health = true
  }
}
