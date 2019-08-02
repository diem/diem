data "template_file" "prometheus_yml" {
  template = file("templates/prometheus.yml")

  vars = {
    workspace             = terraform.workspace
    validator_nodes       = join(",", formatlist("%s:%s", aws_instance.validator.*.private_ip, var.peer_ids))
    validator_svcs        = join(",", formatlist("%s:%s", aws_instance.validator.*.private_ip, var.peer_ids))
    other_nodes           = join(",", ["${aws_instance.prometheus.private_ip}:prometheus", "${aws_instance.faucet.private_ip}:faucet"])
    monitoring_private_ip = aws_instance.prometheus.private_ip
  }
}

data "template_file" "datasources_yml" {
  template = file("templates/grafana-datasources.yml")

  vars = {
    ip = aws_instance.prometheus.private_ip
  }
}


data "template_file" "alertmanager_yml" {
  template = file("templates/alertmanager.yml")

  vars = {
    workspace     = terraform.workspace
    pagerduty_key = var.prometheus_pagerduty_key
  }
}

resource "aws_instance" "prometheus" {
  ami                         = data.aws_ami.ecs.id
  instance_type               = "t3.medium"
  subnet_id                   = element(aws_subnet.testnet.*.id, 0)
  depends_on                  = [aws_main_route_table_association.testnet]
  vpc_security_group_ids      = [aws_security_group.monitoring.id]
  associate_public_ip_address = true
  key_name                    = aws_key_pair.libra.key_name
  iam_instance_profile        = aws_iam_instance_profile.ecsInstanceRole.name
  user_data                   = local.user_data

  tags = {
    Name      = "${terraform.workspace}-prometheus"
    Role      = "prometheus"
    Workspace = terraform.workspace
  }

  provisioner "remote-exec" {
    inline = [
      "sudo mkdir -p /data/prometheus && sudo chown 65534 /data/prometheus",
      "sudo mkdir -p /data/alertmanager && sudo chown 65534 /data/alertmanager",
      "sudo mkdir -p /data/grafana && sudo chown 472 /data/grafana",
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

resource "null_resource" "prometheus" {
  triggers = {
    prometheus_instance = aws_instance.prometheus.id
    prometheus_config   = sha1(data.template_file.prometheus_yml.rendered)
    grafana_datasources = sha1(data.template_file.datasources_yml.rendered)
  }

  provisioner "file" {
    content     = data.template_file.prometheus_yml.rendered
    destination = "/opt/prometheus/prometheus.yml"

    connection {
      host        = aws_instance.prometheus.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "file" {
    source      = "templates/prometheus/"
    destination = "/opt/prometheus"

    connection {
      host        = aws_instance.prometheus.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "local-exec" {
    command = "curl -X POST --max-time 5 --silent http://${aws_instance.prometheus.public_ip}:9090/-/reload || true"
  }

  provisioner "file" {
    content     = data.template_file.alertmanager_yml.rendered
    destination = "/opt/alertmanager/alertmanager.yml"

    connection {
      host        = aws_instance.prometheus.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "file" {
    content     = data.template_file.datasources_yml.rendered
    destination = "/opt/grafana/provisioning/datasources/prometheus.yml"

    connection {
      host        = aws_instance.prometheus.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "file" {
    content     = file("templates/grafana-dashboards.yml")
    destination = "/opt/grafana/provisioning/dashboards/dashboards.yml"

    connection {
      host        = aws_instance.prometheus.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }

  provisioner "file" {
    source      = "templates/dashboards"
    destination = "/opt/grafana/dashboards/libra"

    connection {
      host        = aws_instance.prometheus.public_ip
      user        = "ec2-user"
      private_key = file(var.ssh_priv_key_file)
    }
  }
}

data "template_file" "ecs_prometheus_definition" {
  template = file("templates/prometheus.json")

  vars = {
    prometheus_image   = "prom/prometheus:v2.9.2"
    alertmanager_image = "prom/alertmanager:v0.17.0"
    grafana_image      = "grafana/grafana:latest"
  }
}

resource "aws_ecs_task_definition" "prometheus" {
  family                = "${terraform.workspace}-prometheus"
  container_definitions = data.template_file.ecs_prometheus_definition.rendered
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
    expression = "ec2InstanceId == ${aws_instance.prometheus.id}"
  }

  tags = {
    Role      = "prometheus"
    Workspace = terraform.workspace
  }
}

resource "aws_ecs_service" "prometheus" {
  depends_on                         = [null_resource.prometheus]
  name                               = "${terraform.workspace}-prometheus"
  cluster                            = aws_ecs_cluster.testnet.id
  task_definition                    = aws_ecs_task_definition.prometheus.arn
  desired_count                      = 1
  deployment_minimum_healthy_percent = 0

  tags = {
    Role      = "prometheus"
    Workspace = terraform.workspace
  }
}
