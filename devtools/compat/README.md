# Compat

Tooling to assess forwards and backwards compatibility.

## Install

Using pipenv to manage dependencies:

```
$ ./build.sh
```

## Record and Replay (ROAR)

ROAR CLI tool allows you to take a snapshot of a Libra Network that was spun up in a Terraform workspace. It outputs
a Terraform variable definition (`.tfvars`) file, which can be `terraform apply`ied to "replay" the network. This is
useful to check compatibility of new features at a network level "catch-all" scale.

For example, to replay a Libra Network on a Terraform workspace named `dev`:

```
$ ./run_roar.sh --source dev \
    --tf-dir ../terraform \
    --var-file ../terraform/terraform.tfvars \
    --out-file ../terraform/dev-replay.tfvars
```

This generates a file `dev-replay.tfvars`, which can be applied to a separate workspace:

```
$ terraform workspace select devreplay
$ terraform apply --var-file=dev-replay.tfvars
```
