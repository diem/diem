# Terraform

This directory contains [Terraform](https://www.terraform.io/) configuration to deploy an entire Testnet on Amazon Web Services. It deploys a set of validator nodes (four by default), a faucet server for minting Libra, and a monitoring node running Prometheus.

## Running a Testnet

1. [Download](https://www.terraform.io/downloads.html) and install Terraform.
2. Create an AWS account and setup your access keys.
3. Generate an SSH key (or use an existing one).
4. Edit `terraform.tfvars` and set the following:
  * `api_sources_ipv4`: Set this to the IPs/CIDRs from which you want to access the API endpoints.
  * `ssh_sources_ipv4`/`ssh_sources_ipv6`: Set this to the IPS/CIDRs from which you want to SSH to instances.
  * `ssh_pub_key`: Insert the content of your SSH public key.
  * `ssh_priv_key_file`: Set this to the filename of your SSH private key.
5. Run `terraform init`
6. Run `terraform apply` to startup the system in AWS.
7. In the AWS console go to the EC2 service in your region, and list the Load Balancers. Note the DNS name of the "default-jsonrpc" and "default-faucet" load balancers.
8. Run the client as follows:
  `cargo run -p cli --bin cli -- -a <default-jsonrpc LB DNS name> -p 80 -f <default-faucet LB DNS name>`
