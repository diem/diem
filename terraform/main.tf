terraform {
  required_version = ">= 0.12"
}

provider "aws" {
  version = "~> 2.12"
  region  = var.region
}

data "aws_availability_zones" "available" {
  state             = "available"
  blacklisted_names = ["us-west-2d", "us-east-1e"]
}
