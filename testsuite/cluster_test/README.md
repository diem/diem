**Cluster test** is framework that introduces different failures to system and verifies that some degree of liveliness and safety is preserved during those experiments.

Cluster test works with real AWS cluster and uses root ssh access and AWS api to introduce failures.


###### Structure

Major components of cluster test include:
* **Effect** is a single type of failure, normally affecting single specific node. For example, `Reboot{SomeNode}`
* **Experiment** is some condition we want to test our system with. Usually experiment is set of Effects, for example `RebootRandomValidators` experiment would generate some number of `Reboot` effects for subset of nodes in cluster.
* **HealthCheck** is how we verify whether experiment was successful or not. Examples are `LivenessHealthCheck` that verifies that validators produce commits and `CommitHistoryHealthCheck` that verifies safety, in terms that validators do not produce contradicting commits.

###### Test lifecycle

Normally experiment lifecycle consist of multiple stages.
This lifecycle is managed by test runner:
* Before experiment runner verifies that cluster is healthy
* Experiment is started in separate thread. Experiment has timeout, if it does not finish within this timeout it is considered to be failure
* When `Experiment` is running it also reports set of validators that affected by it through `Experiment::affected_validators()`. We still verify that liveness and safety is not violated for any other validators. For example, when rebooting 3 validators we make sure that all other validators still make progress.
* After experiment completes, we verify that all nodes in cluster becomes healthy again within some timeout

###### Run and build

Normally we run cluster_test on linux machine in AWS. In order to build linux binary on mac laptopt we have cross compilation script:

`docker/cluster_test/build.sh`

This script requires docker for mac and starts docker container with build environment.

Build in this container is incremental, first build takes a lot of time but second build is much faster.
As a result, build script produces binary by default. Running it with `--build-docker-image` will also produce docker image.
