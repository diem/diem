Diem Public Fullnode Deployment
================================

This Helm chart deploys a public fullnode for the Diem blockchain network. The
fullnode connects to Diem validators and synchronises the blockchain state to
a persistent volume. It provides a [JSON-RPC interface][] for interacting with
the blockchain.


Configuration
-------------

See [values.yaml][] for the full list of options you can configure.

* `chain.name`: Select which blockchain to connect to. Current values:
  - "testnet": Diem testnet
* `storage.class`: This needs to be set to a StorageClass available in your
  Kubernetes cluster. Example values:
  - AWS: "gp2"
  - GCP: "standard"
  - Azure: "managed"
* `service.type`: By default the JSON-RPC endpoint is only exposed within the
  Kubernetes cluster. If you want to expose it externally set this to
  "LoadBalancer".
* `service.loadBalancerSourceRanges`: If you enable the LoadBalancer service you
  can set this to a list of IP ranges to restrict access to.
* `image.tag`: Select the image tag to deploy. For a full list of image tags, check out the [Diem dockerhub][]. Backup and restore images are specified separately in `backup.image.tag` and `restore.image.tag`

Connecting to Testnet
-------------

To connect to the Diem testnet, you must have the correct genesis blob and waypoint. The source of truth for these are hosted here:
* https://testnet.diem.com/waypoint.txt
* https://testnet.diem.com/genesis.blob

The waypoint is configured as a helm value in `diem_chains.testnet.waypoint`, and the genesis blob should be copied to `files/genesis/testnet.blob`

Deployment
----------

1. Install Helm v3: https://helm.sh/docs/intro/install/
2. Configure `kubectl` with the Kubernetes cluster you wish to use.
3. Install the release, setting any options:

       $ helm install fullnode --set storage.class=gp2 .


[json-rpc interface]: https://github.com/diem/diem/blob/main/json-rpc/json-rpc-spec.md
[values.yaml]: values.yaml
[Diem dockerhub]: https://hub.docker.com/r/diem/validator/tags?page=1&ordering=last_updated
