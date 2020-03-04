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

variable "num_validators_in_genesis" {
  default     = 4
  description = "Number of validator nodes to include in genesis blob"
}

# This allows you to use a override number of validators for config generation
variable "cfg_num_validators_override" {
  default     = 0
  description = "Number of validators to use when generating configs, 0 will default to using num_validators"
}

variable "num_fullnodes" {
  default     = 1
  description = "Number of full nodes to run per fullnode network"
}

variable "num_fullnode_networks" {
  default     = 1
  description = "Number of full nodes networks to run (must be <= num_validators)"
}

variable "fullnode_seed" {
  default     = 2674267426742674267426742674267426742674267426742674267426742674
  description = "Default seed for fullnode network"
}

variable "validator_type" {
  description = "EC2 instance type of validator instances"
  default     = "c5.large"
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

variable "log_to_file" {
  type        = bool
  default     = false
  description = "Set to true to log to /opt/libra/data/libra.log (in container) and /data/libra/libra.log (on host). This file won't be log rotated, you need to handle log rotation on your own if you choose this option"
}

variable "log_path" {
  type    = string
  default = "/opt/libra/data/libra.log"
}

variable "enable_logstash" {
  type        = bool
  description = "Enable logstash instance on validator to send logs to elasticservice, this will enable log_to_file"
  default     = false
}

variable "logstash_image" {
  type    = string
  default = ""
}

variable "logstash_version" {
  type    = string
  default = "latest"
}

variable "elastic_storage_size" {
  default     = 500
  description = "The volume size for Elasticsearch"
}

variable "safety_rules_image_repo" {
  type        = string
  description = "Docker image repository to use for safety-rules"
  default     = "docker.libra.org/safety-rules"
}

variable "safety_rules_image_tag" {
  type        = string
  description = "Docker image tag to use for safety-rules"
  default     = "latest"
}

variable "restore_vol_id" {
  default     = ""
  description = "volume id to restore validator data from"
}
