data "template_file" "prometheus_yml" {
  template = file("templates/prometheus.yml")

  vars = {
    workspace             = terraform.workspace
    validators            = join(",", formatlist("%s:%s", aws_instance.validator.*.private_ip, range(var.num_validators)))
    fullnodes             = join(",", formatlist("%s:%s", aws_instance.fullnode.*.private_ip, range(var.num_fullnodes)))
    other_nodes           = join(",", ["${aws_instance.monitoring.private_ip}:monitoring", "${aws_instance.faucet.private_ip}:faucet", "${aws_instance.vault.private_ip}:vault"])
    vault_ip              = aws_instance.vault.private_ip
    monitoring_private_ip = aws_instance.monitoring.private_ip
  }
}

data "template_file" "datasources_yml" {
  template = file("templates/grafana-datasources.yml")

  vars = {
    ip = aws_instance.monitoring.private_ip
  }
}


data "template_file" "alertmanager_yml" {
  template = file("templates/alertmanager.yml")

  vars = {
    workspace     = terraform.workspace
    pagerduty_key = var.prometheus_pagerduty_key
  }
}

resource "aws_instance" "monitoring" {
  ami                         = local.aws_ecs_ami
  instance_type               = "t3.medium"
  subnet_id                   = element(aws_subnet.testnet.*.id, 0)
  depends_on                  = [aws_main_route_table_association.testnet]
  vpc_security_group_ids      = [aws_security_group.monitoring.id]
  associate_public_ip_address = true
  key_name                    = aws_key_pair.libra.key_name
  iam_instance_profile        = aws_iam_instance_profile.ecsInstanceRole.name
  user_data                   = local.user_data

  tags = {
    Name      = "${terraform.workspace}-monitoring"
    Role      = "monitoring"
    Terraform = "testnet"
    Workspace = terraform.workspace
  }

  provisioner "remote-exec" {
    inline = [
      "sudo mkdir -p /opt/{prometheus,alertmanager}",
      "sudo mkdir -p /opt/grafana/dashboards",
      "sudo mkdir -p /opt/grafana/provisioning/{datasources,dashboards,notifiers}",
      "sudo chown -R ec2-user /opt/{prometheus,alertmanager,grafana}",
    ]

    connection {
      host        = coalesce(self.public_ip, self.private_ip)
      type        = "ssh"
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }
}

resource "aws_ebs_volume" "monitoring" {
  availability_zone = data.aws_availability_zones.available.names[0]
  size              = var.monitoring_ebs_volume
  type              = "standard"
  snapshot_id       = var.monitoring_snapshot

  tags = {
    Name      = "${terraform.workspace}-monitoring"
    Terraform = "testnet"
    Workspace = terraform.workspace
    Role      = "monitoring"
  }

  lifecycle {
    ignore_changes = [snapshot_id, size]
  }
}

resource "aws_volume_attachment" "monitoring" {
  instance_id = aws_instance.monitoring.id
  volume_id   = aws_ebs_volume.monitoring.id
  device_name = "/dev/xvdb"

  provisioner "remote-exec" {
    inline = [
      "if ! sudo file -s /dev/nvme1n1 | grep -q filesystem; then sudo mkfs.ext4 /dev/nvme1n1; fi",
      "echo '/dev/nvme1n1 /data ext4 defaults,noatime 0 2' | sudo tee -a /etc/fstab",
      "sudo mkdir -p /data; sudo mount /data || true",
      "sudo mkdir -p /data/prometheus && sudo chown 65534 /data/prometheus",
      "sudo mkdir -p /data/alertmanager && sudo chown 65534 /data/alertmanager",
      "sudo mkdir -p /data/grafana && sudo chown 472 /data/grafana",
    ]

    connection {
      host        = aws_instance.monitoring.public_ip
      type        = "ssh"
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }
}

resource "null_resource" "monitoring" {
  triggers = {
    monitoring_instance = aws_instance.monitoring.id
    prometheus_config   = sha1(data.template_file.prometheus_yml.rendered)
    grafana_datasources = sha1(data.template_file.datasources_yml.rendered)
  }

  provisioner "file" {
    content     = data.template_file.prometheus_yml.rendered
    destination = "/opt/prometheus/prometheus.yml"

    connection {
      host        = aws_instance.monitoring.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "file" {
    source      = "templates/prometheus/"
    destination = "/opt/prometheus"

    connection {
      host        = aws_instance.monitoring.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "local-exec" {
    command = "curl -X POST --max-time 5 --silent http://${aws_instance.monitoring.public_ip}:9090/-/reload || true"
  }

  provisioner "file" {
    content     = data.template_file.alertmanager_yml.rendered
    destination = "/opt/alertmanager/alertmanager.yml"

    connection {
      host        = aws_instance.monitoring.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "file" {
    content     = data.template_file.datasources_yml.rendered
    destination = "/opt/grafana/provisioning/datasources/prometheus.yml"

    connection {
      host        = aws_instance.monitoring.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "file" {
    content     = file("templates/grafana-dashboards.yml")
    destination = "/opt/grafana/provisioning/dashboards/dashboards.yml"

    connection {
      host        = aws_instance.monitoring.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "file" {
    source      = "templates/dashboards"
    destination = "/opt/grafana/dashboards/libra"

    connection {
      host        = aws_instance.monitoring.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }
}

data "template_file" "ecs_monitoring_definition" {
  template = file("templates/monitoring.json")

  vars = {
    prometheus_image   = "prom/prometheus:v2.9.2"
    pushgateway_image  = "prom/pushgateway:v1.2.0"
    alertmanager_image = "prom/alertmanager:v0.17.0"
    grafana_image      = "grafana/grafana:6.7.3"
  }
}

resource "aws_ecs_task_definition" "monitoring" {
  family                = "${terraform.workspace}-monitoring"
  container_definitions = data.template_file.ecs_monitoring_definition.rendered
  execution_role_arn    = aws_iam_role.ecsTaskExecutionRole.arn

  volume {
    name      = "prometheus-data"
    host_path = "/data/prometheus"
  }

  volume {
    name      = "prometheus-config"
    host_path = "/opt/prometheus"
  }

  volume {
    name      = "alertmanager-data"
    host_path = "/data/alertmanager"
  }

  volume {
    name      = "alertmanager-config"
    host_path = "/opt/alertmanager"
  }

  volume {
    name      = "grafana-data"
    host_path = "/data/grafana"
  }

  volume {
    name      = "grafana-provisioning"
    host_path = "/opt/grafana/provisioning"
  }

  volume {
    name      = "grafana-dashboards"
    host_path = "/opt/grafana/dashboards"
  }

  placement_constraints {
    type       = "memberOf"
    expression = "ec2InstanceId == ${aws_instance.monitoring.id}"
  }

  tags = {
    Role      = "monitoring"
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
}

resource "aws_ecs_service" "monitoring" {
  depends_on                         = [null_resource.monitoring, aws_volume_attachment.monitoring]
  name                               = "${terraform.workspace}-monitoring"
  cluster                            = aws_ecs_cluster.testnet.id
  task_definition                    = aws_ecs_task_definition.monitoring.arn
  desired_count                      = 1
  deployment_minimum_healthy_percent = 0

  tags = {
    Role      = "monitoring"
    Terraform = "testnet"
    Workspace = terraform.workspace
  }
}
