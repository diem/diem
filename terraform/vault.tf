resource "aws_kms_key" "vault" {
  description             = "Vault unseal key"
  deletion_window_in_days = 7

  tags = {
    Name = "${terraform.workspace}-vault"
  }
}

resource "aws_instance" "vault" {
  ami                         = local.aws_ecs_ami
  instance_type               = var.vault_type
  subnet_id                   = aws_subnet.testnet[0].id
  depends_on                  = [aws_main_route_table_association.testnet, aws_iam_role_policy.ecs_extra]
  vpc_security_group_ids      = [aws_security_group.vault.id]
  associate_public_ip_address = local.instance_public_ip
  key_name                    = aws_key_pair.libra.key_name
  iam_instance_profile        = aws_iam_instance_profile.ecsInstanceRole.name
  user_data                   = local.user_data

  tags = {
    Name      = "${terraform.workspace}-vault"
    Role      = "vault"
    Workspace = terraform.workspace
  }
}

resource "aws_ecs_task_definition" "vault" {
  family             = "${terraform.workspace}-vault"
  execution_role_arn = aws_iam_role.ecsTaskExecutionRole.arn
  network_mode       = "host"

  container_definitions = jsonencode([
    {
      name      = "vault",
      image     = "vault:latest",
      command   = ["server"],
      cpu       = 2000,
      memory    = 2048,
      essential = true,
      portMappings = [
        { containerPort = 8200, hostPort = 8200 },
      ],
      mountPoints = [
        { sourceVolume = "vault-data", containerPath = "/vault/file" },
      ],
      environment = [
        { name = "VAULT_LOCAL_CONFIG", value = jsonencode({
          backend = {
            file = { path = "/vault/file" }
          },
          listener = {
            tcp = {
              address     = "0.0.0.0:8200",
              tls_disable = "true",
            },
          },
          seal = {
            awskms = { kms_key_id = aws_kms_key.vault.id }
          },
        }) },
      ],
      linuxParameters = {
        capabilities = {
          add = ["IPC_LOCK"],
        },
      },
    }, {
      name       = "vault-init",
      image      = "vault:latest",
      entryPoint = ["sh"],
      command    = ["-c", "sleep 5s && TOKEN=$(vault operator init | grep 'Root Token' | cut -d: -f2) && vault login $TOKEN && vault token create -id=root -policy=root && vault secrets enable -path=secret kv-v2"],
      cpu        = 40,
      memory     = 128,
      essential  = false,
      environment = [
        { name = "VAULT_ADDR", value = "http://localhost:8200" },
      ],
    },
  ])

  volume {
    name      = "vault-data"
    host_path = "/vault"
  }

  placement_constraints {
    type       = "memberOf"
    expression = "ec2InstanceId == ${aws_instance.vault.id}"
  }

  tags = {
    Role      = "vault"
    Workspace = terraform.workspace
  }
}

resource "aws_ecs_service" "vault" {
  name                               = "${terraform.workspace}-vault"
  cluster                            = aws_ecs_cluster.testnet.id
  task_definition                    = aws_ecs_task_definition.vault.arn
  desired_count                      = 1
  deployment_minimum_healthy_percent = 0

  tags = {
    Role      = "vault"
    Workspace = terraform.workspace
  }
}
