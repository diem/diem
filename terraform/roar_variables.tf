variable "mix_validators" {
  type    = bool
  default = false
  description = "Mix validator images for record and replay test"
}

variable "mix_image_tags" {
  type        = list(string)
  default     = ["latest_dynamic"]
  description = "List of Docker image tags to be used in record and replay test"
}
