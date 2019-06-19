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
  default     = ["0.0.0.0/0"]
}

variable "ssh_sources_ipv6" {
  type        = list(string)
  description = "List of IPv6 CIDR blocks from which to allow SSH access"
  default     = ["::/0"]
}

variable "api_sources_ipv4" {
  type        = list(string)
  description = "List of IPv4 CIDR blocks from which to allow API access"
  default     = ["0.0.0.0/0"]
}

variable "image_repo" {
  type        = string
  description = "Docker image repository to use for validator"
}

variable "image_tag" {
  type        = string
  description = "Docker image tag to use for validator"
  default     = "latest"
}

variable "peer_ids" {
  type        = list(string)
  description = "List of PeerIds"
}

variable "validator_type" {
  description = "EC2 instance type of validator instances"
  default     = "m5.large"
}

variable "faucet_image_repo" {
  description = "Docker image repository to use for faucet server"
}

variable "faucet_log_level" {
  description = "Log level for faucet to pass to gunicorn"
  default     = "info"
}

variable "faucet_image_tag" {
  description = "Docker image tag to use for faucet server"
  default     = "latest"
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

variable "append_workspace_dns" {
  description = "Append Terraform workspace to DNS names created"
  default     = true
}

variable "prometheus_pagerduty_key" {
  default     = ""
  description = "Key for Prometheus-PagerDuty integration" 
}
