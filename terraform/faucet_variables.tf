variable "faucet_image_repo" {
  description = "Docker image repository to use for faucet server"
  default     = "docker.libra.org/faucet"
}

variable "faucet_log_level" {
  description = "Log level for faucet to pass to gunicorn"
  default     = "info"
}

variable "faucet_image_tag" {
  description = "Docker image tag to use for faucet server"
  default     = "latest_dynamic"
}
