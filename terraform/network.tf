resource "aws_vpc" "testnet" {
  cidr_block                       = "10.0.0.0/16"
  assign_generated_ipv6_cidr_block = true
  enable_dns_hostnames             = true

  tags = {
    Name = terraform.workspace
  }
}

resource "aws_subnet" "testnet" {
  count                           = length(data.aws_availability_zones.available.names)
  vpc_id                          = aws_vpc.testnet.id
  cidr_block                      = cidrsubnet(aws_vpc.testnet.cidr_block, 6, count.index)
  ipv6_cidr_block                 = cidrsubnet(aws_vpc.testnet.ipv6_cidr_block, 8, count.index)
  assign_ipv6_address_on_creation = true
  availability_zone               = data.aws_availability_zones.available.names[count.index]
  map_public_ip_on_launch         = true

  tags = {
    Name = "${terraform.workspace}-${data.aws_availability_zones.available.names[count.index]}"
  }
}

resource "aws_internet_gateway" "testnet" {
  vpc_id = aws_vpc.testnet.id

  tags = {
    Name = terraform.workspace
  }
}

resource "aws_route_table" "testnet" {
  vpc_id = aws_vpc.testnet.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.testnet.id
  }

  route {
    ipv6_cidr_block = "::/0"
    gateway_id      = aws_internet_gateway.testnet.id
  }

  tags = {
    Name = terraform.workspace
  }
}

resource "aws_main_route_table_association" "testnet" {
  vpc_id         = aws_vpc.testnet.id
  route_table_id = aws_route_table.testnet.id
}

resource "aws_security_group" "monitoring" {
  name        = "${terraform.workspace}-monitoring"
  description = "Monitoring services"
  vpc_id      = aws_vpc.testnet.id
}

resource "aws_security_group_rule" "monitoring-ssh" {
  security_group_id = aws_security_group.monitoring.id
  type              = "ingress"
  from_port         = 22
  to_port           = 22
  protocol          = "tcp"
  cidr_blocks       = var.ssh_sources_ipv4
  ipv6_cidr_blocks  = var.ssh_sources_ipv6
}

resource "aws_security_group_rule" "monitoring-prometheus" {
  security_group_id = aws_security_group.monitoring.id
  type              = "ingress"
  from_port         = 9090
  to_port           = 9091
  protocol          = "tcp"
  cidr_blocks       = var.ssh_sources_ipv4
  ipv6_cidr_blocks  = var.ssh_sources_ipv6
}

resource "aws_security_group_rule" "monitoring-pushgateway" {
  security_group_id        = aws_security_group.monitoring.id
  type                     = "ingress"
  from_port                = 9092
  to_port                  = 9092
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.validator.id
}

resource "aws_security_group_rule" "monitoring-egress" {
  security_group_id = aws_security_group.monitoring.id
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  ipv6_cidr_blocks  = ["::/0"]
}

resource "aws_security_group" "validator" {
  name        = "${terraform.workspace}-validator"
  description = "Validator node"
  vpc_id      = aws_vpc.testnet.id
}

resource "aws_security_group_rule" "validator-ssh" {
  security_group_id = aws_security_group.validator.id
  type              = "ingress"
  from_port         = 22
  to_port           = 22
  protocol          = "tcp"
  cidr_blocks       = var.ssh_sources_ipv4
  ipv6_cidr_blocks  = var.ssh_sources_ipv6
}

resource "aws_security_group_rule" "validator-node" {
  security_group_id = aws_security_group.validator.id
  type              = "ingress"
  from_port         = 6180
  to_port           = 6181
  protocol          = "tcp"
  cidr_blocks       = concat(var.validator_node_sources_ipv4, [aws_vpc.testnet.cidr_block])
  ipv6_cidr_blocks  = concat(var.validator_node_sources_ipv6, [aws_vpc.testnet.ipv6_cidr_block])
}

resource "aws_security_group_rule" "validator-ac" {
  security_group_id = aws_security_group.validator.id
  type              = "ingress"
  from_port         = 8000
  to_port           = 8001
  protocol          = "tcp"
  cidr_blocks       = concat(var.api_sources_ipv4, [aws_vpc.testnet.cidr_block])
}

resource "aws_security_group_rule" "validator-jsonrpc" {
  security_group_id = aws_security_group.validator.id
  type              = "ingress"
  from_port         = 8080
  to_port           = 8080
  protocol          = "tcp"
  cidr_blocks       = [aws_vpc.testnet.cidr_block]
}

resource "aws_security_group" "jsonrpc-lb" {
  name        = "${terraform.workspace}-jsonrpc-lb"
  description = "jsonrpc-lb"
  vpc_id      = aws_vpc.testnet.id
}

resource "aws_security_group_rule" "json-rpc" {
  security_group_id = aws_security_group.jsonrpc-lb.id
  type              = "ingress"
  from_port         = 80
  to_port           = 80
  protocol          = "tcp"
  cidr_blocks       = concat(var.api_sources_ipv4, [aws_vpc.testnet.cidr_block])
}

resource "aws_security_group_rule" "json-rpc-https" {
  security_group_id = aws_security_group.jsonrpc-lb.id
  type              = "ingress"
  from_port         = 443
  to_port           = 443
  protocol          = "tcp"
  cidr_blocks       = concat(var.api_sources_ipv4, [aws_vpc.testnet.cidr_block])
}

resource "aws_security_group_rule" "jsonrpc-lb-egress" {
  security_group_id = aws_security_group.jsonrpc-lb.id
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  ipv6_cidr_blocks  = ["::/0"]
}

resource "aws_security_group_rule" "validator-host-mon" {
  security_group_id        = aws_security_group.validator.id
  type                     = "ingress"
  from_port                = 9100
  to_port                  = 9101
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.monitoring.id
}

resource "aws_security_group_rule" "validator-egress" {
  security_group_id = aws_security_group.validator.id
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  ipv6_cidr_blocks  = ["::/0"]
}



resource "aws_security_group" "faucet-host" {
  name        = "${terraform.workspace}-faucet-host"
  description = "faucet-host"
  vpc_id      = aws_vpc.testnet.id
}

resource "aws_security_group_rule" "faucet-host-egress" {
  security_group_id = aws_security_group.faucet-host.id
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  ipv6_cidr_blocks  = ["::/0"]
}

resource "aws_security_group_rule" "faucet-host-ssh" {
  security_group_id = aws_security_group.faucet-host.id
  type              = "ingress"
  from_port         = 22
  to_port           = 22
  protocol          = "tcp"
  cidr_blocks       = var.ssh_sources_ipv4
  ipv6_cidr_blocks  = var.ssh_sources_ipv6
}

resource "aws_security_group_rule" "faucet-host-application-private" {
  security_group_id = aws_security_group.faucet-host.id
  type              = "ingress"
  from_port         = 8000
  to_port           = 8000
  protocol          = "tcp"
  cidr_blocks       = [aws_vpc.testnet.cidr_block]
}

resource "aws_security_group_rule" "faucet-host-mon" {
  security_group_id        = aws_security_group.faucet-host.id
  type                     = "ingress"
  from_port                = 9100
  to_port                  = 9100
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.monitoring.id
}

resource "aws_security_group" "faucet-lb" {
  name        = "${terraform.workspace}-faucet-lb"
  description = "faucet-lb"
  vpc_id      = aws_vpc.testnet.id
}

resource "aws_security_group_rule" "faucet-lb-egress" {
  security_group_id = aws_security_group.faucet-lb.id
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  ipv6_cidr_blocks  = ["::/0"]
}

resource "aws_security_group_rule" "faucet-lb-application" {
  security_group_id = aws_security_group.faucet-lb.id
  type              = "ingress"
  from_port         = 80
  to_port           = 80
  protocol          = "tcp"
  cidr_blocks       = var.api_sources_ipv4
}

resource "aws_security_group" "vault" {
  name        = "${terraform.workspace}-vault"
  description = "Vault secrets manager"
  vpc_id      = aws_vpc.testnet.id
}

resource "aws_security_group_rule" "vault-validator" {
  security_group_id        = aws_security_group.vault.id
  type                     = "ingress"
  from_port                = 8200
  to_port                  = 8200
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.validator.id
}

resource "aws_security_group_rule" "vault-ssh" {
  security_group_id = aws_security_group.vault.id
  type              = "ingress"
  from_port         = 22
  to_port           = 22
  protocol          = "tcp"
  cidr_blocks       = var.ssh_sources_ipv4
  ipv6_cidr_blocks  = var.ssh_sources_ipv6
}

resource "aws_security_group_rule" "vault-mon-host" {
  security_group_id        = aws_security_group.vault.id
  type                     = "ingress"
  from_port                = 9100
  to_port                  = 9100
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.monitoring.id
}

resource "aws_security_group_rule" "vault-mon-vault" {
  security_group_id        = aws_security_group.vault.id
  type                     = "ingress"
  from_port                = 8200
  to_port                  = 8200
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.monitoring.id
}

resource "aws_security_group_rule" "vault-egress" {
  security_group_id = aws_security_group.vault.id
  type              = "egress"
  from_port         = 0
  to_port           = 0
  protocol          = -1
  cidr_blocks       = ["0.0.0.0/0"]
  ipv6_cidr_blocks  = ["::/0"]
}
