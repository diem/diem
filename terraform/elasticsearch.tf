resource "aws_elasticsearch_domain" "logging" {
  domain_name           = "${terraform.workspace}-logging"
  elasticsearch_version = "7.1"
  count                 = var.enable_logstash ? 1 : 0

  cluster_config {
    instance_type = "r5.large.elasticsearch"
  }

  ebs_options {
      ebs_enabled = true
      volume_size = 20
  }

  snapshot_options {
    automated_snapshot_start_hour = 23
  }
}

data "aws_iam_policy_document" "logging" {
  count     = var.enable_logstash ? 1 : 0
  statement {
    actions = [
      "es:*",
    ]

    resources = [
      "${element(aws_elasticsearch_domain.logging.*.arn, count.index)}/*",
    ]

    principals {
      type        = "*"
      identifiers = ["*"]
    }

    condition {
      test     = "IpAddress"
      variable = "aws:SourceIp"

      values = concat(["199.201.64.0/22",],aws_instance.validator.*.public_ip)
    }
  }
}

resource "aws_elasticsearch_domain_policy" "main" {
  count       = var.enable_logstash ? 1 : 0
  domain_name = element(aws_elasticsearch_domain.logging.*.domain_name, count.index)

  access_policies = element(data.aws_iam_policy_document.logging.*.json, count.index)
}
