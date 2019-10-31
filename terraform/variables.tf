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
  default     = "latest"
}

variable "peer_ids" {
  type        = list(string)
  description = "List of validator PeerIds"
}

variable "fullnode_ids" {
  type        = list(string)
  description = "List of full node PeerIds"
}

variable "validator_fullnode_id" {
  type        = string
  description = "PeerId of the validator on the full node network"
}

variable "num_fullnodes" {
  default     = 1
  description = "Number of full nodes to run (attached to first validator)"
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

variable "validator_set" {
  description = "Relative path to directory containing validator set configs"
  default     = "validator-sets/dev"
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
