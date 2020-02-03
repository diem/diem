# Libra Configuration

The Libra Configuration describes the operational details for a Libra Node
(Validator or Full node) and provides the Libra Clients information on how to
connect to the blockchain and derive trust.

A Validator performs the BFT protocol and hosts the source of truth for the
blockchain.

A Full node offers replication services for a Validator and provides an
additional entry point for Libra Clients to submit read requests (i.e.,
queries about the state of the blockchain). This reduces the operational
burden on Validators.

Clients are any service interested in learning about the state of the
blockchain or performing transactions.

For a more detailed summary of the differences between a Validator and a Full
node, see this [blog post](https://developers.libra.org/blog/2020/01/23/full-node-basics).

## Organization

Libra Configuration is broken up into many utilities:
- `src/config` hosts the core configuration file structure
- `src/generator.rs` assists in building sets of configurations for a Validator
  or Full node set
- `src/keys.rs` specifies means for storing and accessing keys from within
  configurations
- `config-builder` extends `src/generator.rs` with a command-line utility
  and also provides support for generating genesis
- `generate-keypair` generates Ed25519 key pairs in Libra Canonical
  Serialization (LCS) format

The separation of the `config-builder` into its own crate was dictated by the
need for `config-builder` to be able to generate genesis. Genesis requires the
VM as a dependency, which is not a dependency for loading or using the
configuration from many of the services.

## Building a Test Network (TestNet)

`config-builder` builds a single node's configuration within a network or swarm
of nodes. This can be used to create a Libra TestNet or to add a Full node to an
existing network. In addition, it enables generation of a mint/faucet client
capable of performing mint transactions/creating accounts.

## Generating a new TestNet

The only requirements for generating a TestNet configuration are: (i) having a
list of IP addresses and ports to host each Libra Node; (ii) a pre-agreed upon
shared secret; and (iii) a fixed ordering for each Node in the network. Full
node networks can either be added to existing configs or generate completely
new configs.

Each peer, `I`, can then generate their own configurations by:

    config-builder validator \
        -a $PUBLIC_MULTIADDR_FOR_NODE_I \
        -b $PUBLIC_MULTIADDR_FOR_NODE_0 \
        -d /opt/libra/data \
        -i $I \
        -l $ANY_MULTIADDR_FOR_NODE_I \
        -n $TOTAL_NUMBER_OF_NODES \
        -o /opt/libra/etc \
        -s $SHARED_SECRET

As an example, this is the 2nd node (offset 1) in a set of 4:

    config-builder validator \
        -a "/ip4/1.1.1.2/tcp/7000" \
        -b "/ip4/1.1.1.1/tcp/7000" \
        -d /opt/libra/data \
        -i 1 \
        -l "/ip4/0.0.0.0/tcp/7000" \
        -n 4 \
        -o /opt/libra/etc \
        -s 0123456789abcdef101112131415161718191a1b1c1d1e1f2021222324252627

To create a mint service's key:

    config-builder faucet \
        -o /opt/libra/etc \
        -s 0123456789abcdef101112131415161718191a1b1c1d1e1f2021222324252627

Adding a Full node network is similar to instantiating a Validator config.
Though there are three possible routes: (i) creating a new node config; (ii)
extending an existing Full node with another network; and (iii) extending a
Validator with a Full node network. The input is similar for all three cases
with only the command (create vs. extend) differing between them. When
extending a Validator, `config-builder` assumes that there are n + 1 Full
nodes and gives the n + 1 identity to the Validator. This is also the same
peer id set in state sychronization for pure Full nodes. Note: currently, the
tool does not support the creation of trees of Full node networks.

    config-builder full-node (create | extend) \
        -a $PUBLIC_MULTIADDR_FOR_NODE_I \
        -b $PUBLIC_MULTIADDR_FOR_NODE_0 \
        -d /opt/libra/data \
        -l $ANY_MULTIADDR_FOR_NODE_I \
        -n $TOTAL_NUMBER_OF_NODES \
        -o /opt/libra/etc \
        -s $SHARED_SECRET \
        [ -i $I -f $TOTAL_NUMBER_OF_FULL_NODES -c $FULL_NODE_SHARED_SECRET | -p ]

Here's an example of adding a 4 membered and authenticated network connecting to the
node above:

    config-builder full-node create \
        -a "/ip4/1.1.1.2/tcp/7100" \
        -b "/ip4/1.1.1.2/tcp/7100" \
        -d /opt/libra/fn/data \
        -l "/ip4/0.0.0.0/tcp/7100" \
        -n 4 \
        -o /opt/libra/fn/etc \
        -s 0123456789abcdef101112131415161718191a1b1c1d1e1f2021222324252627 \
        -i 0 \
        -f 4 \
        -c 28292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f4041424344454547

Similarly a public network could be added via:

    config-builder full-node create \
        -a "/ip4/1.1.1.2/tcp/7100" \
        -b "/ip4/1.1.1.2/tcp/7100" \
        -d /opt/libra/fn/data \
        -l "/ip4/0.0.0.0/tcp/7100" \
        -n 4 \
        -o /opt/libra/etc \
        -s 0123456789abcdef101112131415161718191a1b1c1d1e1f2021222324252627 \
        -p

## Internals

There are several different configurations contained within Libra Configuration.

### Libra Node Configuration
The Libra Configuration contains several modules:

- AdmissionControlConfig - Where a Libra Node hosts their AC
- BaseConfig - Specifies the Node's role and base directories
- ConsensusConfig - The behaviors of Consensus including the SafetyRules TCB
- DebugInterface - A special gRPC service for identifying internals of the
  system
- ExecutionConfig - The gRPC service endpoint and path to the genesis file
  that defines the Move standard library and the initial Validator set.
- LoggerConfig - Configures the Libra logging service
- MempoolConfig - Parameters for configuring uncommitted transaction storage
- MetricsConfig - Local storage for metrics
- NetworkConfig - LibraNet configuration file that specifies peers with keys,
  seed addresses to connect to upstream peers, the local peers network keys,
and other network configuration parameters
- NodeConfig - Hosts all configuration files for a Libra Node
- SafetyRulesConfig - Specifies the persistency strategy for Libra Safety
  Rules
- StateSyncConfig - Specifies parameters around state sycnhronization and the
  set of peers that provide the data
- StorageConfig - Where the LibraDB is stored and its gRPC service endpoints
- VMConfig - Specifies the permitted publishing options and scripts that can be
  executed

### Client Configuration

- ConsensusPeersConfig - The clients source of truth for the set of Libra
  nodes (Validators and Full nodes derive this from genesis / blockchain).

### Shared Configuration

- TestConfig - Used during tests to hold account keys and temporary paths for
  the configurations that will automatically be cleaned up after the test.

## Testing
Configuration tests serve several purposes:

- Identifying when defaults have changed
- Ensuring that parsing / storing works consistently
- Verifying that default filename assumptions are maintained

Several of the defaults in the configurations, in particular paths and
addresses, have dependecies outside the Libra code base. These tests serve as
a reminder that there may be rammifications from breaking these tests, which
may impact production deployments.

The test configs currently live in `src/config/test_data`.

## TODO

- Add the ability to turn off services that are optional (e.g., debug
  interface, AC gRPC)
- Eliminate ConsensusPeersConfig from ConsensusConfig and make it top level.
- Is the execution gRPC interface being deprecated? Cleanup configs.
- LoggerConfig should allow specifying the severity.
- Eliminate generate-keypair
- Generating public networks is broken
- Make SafetyRule use on disk by default and remove from config generator
