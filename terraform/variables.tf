variable "region" {
  default = "us-west-2"
}

variable "ssh_pub_key" {
  type        = string
  description = "SSH public key for EC2 instance access"
}

variable "ssh_priv_key_file" {
  type        = string
  description = "Filename of SSH private key for EC2 instance access"
}

variable "ssh_sources_ipv4" {
  type        = list(string)
  description = "List of IPv4 CIDR blocks from which to allow SSH access"
}

variable "ssh_sources_ipv6" {
  type        = list(string)
  description = "List of IPv6 CIDR blocks from which to allow SSH access"
}

variable "api_sources_ipv4" {
  type        = list(string)
  description = "List of IPv4 CIDR blocks from which to allow API access"
}

variable "image_repo" {
  type        = string
  description = "Docker image repository to use for validator"
  default     = "docker.libra.org/validator"
}

variable "image_tag" {
  type        = string
  description = "Docker image tag to use for validator"
  default     = "latest_dynamic"
}

# This var is used by cluster test in cluster.rs, please update in both places if this value changes
variable "config_seed" {
  default     = 1337133713371337133713371337133713371337133713371337133713371337
  description = "Seed to be used by libra-config for"
}

variable "num_validators" {
  default     = 4
  description = "Number of validator nodes to run on this network"
}

# This allows you to use a override number of validators for config generation
variable "cfg_num_validators_override" {
  default     = 0
  description = "Number of validators to use when generating configs, 0 will default to using num_validators"
}

variable "num_fullnodes" {
  default     = 1
  description = "Number of full nodes to run on validators"
}

variable "fullnode_distribution" {
  type        = list(number)
  default     = [1, 0, 0, 0]
  description = "List of number of fullnodes on each validator"
}

# This is to generate a list of fullnode with validator index to indicate
# which validator they should be connected to
locals {
  validator_index = range(0, length(var.fullnode_distribution))
  fullnode_pair = zipmap(local.validator_index, var.fullnode_distribution)
  expanded_fullnodes = {
    for key, val in local.fullnode_pair : key => [
      for i in range(val) : format("%d", key)
    ]
  }
  fullnode_list = flatten(values(local.expanded_fullnodes))
}

variable "validator_type" {
  description = "EC2 instance type of validator instances"
  default     = "c5d.large"
}

variable "validator_ebs_size" {
  description = "Size of validator EBS volume in GB"
  default     = 30
}

variable "zone_id" {
  description = "Route53 ZoneId to create records in"
  default     = ""
}

variable "validator_log_level" {
  description = "Log level for validator processes (set with RUST_LOG)"
  default     = "debug"
}

variable "validator_linux_capabilities" {
  type        = list(string)
  description = "List of capabilities needed as Linux parameters"
  default     = []
}

variable "validator_node_sources_ipv4" {
  type        = list(string)
  description = "List of IPv4 CIDR blocks from which to allow Validator Node access"
  default     = []
}

variable "validator_node_sources_ipv6" {
  type        = list(string)
  description = "List of IPv6 CIDR blocks from which to allow Validator Node access"
  default     = []
}

variable "validator_use_public_ip" {
  type    = bool
  default = false
}

variable "append_workspace_dns" {
  description = "Append Terraform workspace to DNS names created"
  default     = true
}

variable "prometheus_pagerduty_key" {
  default     = ""
  description = "Key for Prometheus-PagerDuty integration"
}

variable "monitoring_snapshot" {
  default     = ""
  description = "EBS snapshot ID to initialise monitoring data with"
}

variable "cloudwatch_logs" {
  description = "Send container logs to CloudWatch"
  default     = false
}

variable "monitoring_ebs_volume" {
  default     = 100
  description = "Size of monitoring instance EBS volume in GB"
}
