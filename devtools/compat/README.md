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

## Compatibility Networks (CompatNet/Compat)

Compat CLI tool allows administration and usage of long-running Libra Network instances for mixed-version validator testing. Assuming
there are a pool of ECS clusters named `compat-*`, Compat allows a process to lock a cluster, and performs specified upgrades.

For example, to get access to a network where half of the validators run testnet revision and half run `master_65e3b04`:

```
$ ./run_compat.sh test --tag master_65e3b04 testnet
```

Output of the above command will indicate which cluster it was able to lock, or to retry if it times out. Assuming we have locked a cluster
named `compat-0`, we can run an arbitrary workload against it:

```
$ cargo run -p cli -- --url https://client.compat-0.aws.hlw3truzy4ls.com/ --waypoint <waypoint>
...
> account create
> account mint 0 100 LBR
> query balance 0
```

To clean up, unlock the cluster: `./run_compat.sh admin unlock compat-lock-0`
