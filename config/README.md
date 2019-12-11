# Libra Configuration

The Libra Configuration describes the operational details for a Libra Node
(Validator / Full node) and provides the Libra Clients information on how to
connect to the BlockChain and derive trust.

A Validator performs the BFT protocol and hosts the source of truth for the
blockchain.

A Fullnode offers replication services for a Validator and often the entry
point for clients to limit client queries that may impact the performance of
the blockchain.

Clients are any service interested in learning about the state of the
blockchain or performing transactions.

## Organization

Libra Configuration is broken up into many utilities:
- src/config hosts the core configuration file structure
- src/generation assists in building sets of configurations for a validator /
  full node set
- src/keys specifies means for storing and accessing keys from within
  configurations
- config-builder extends src/generation with a Rust and command-line utility
  and support for generating genesis
- generate-keypair generates Ed25519 key pairs in LCS format

The separation of the config-builder into its own crate was dicated by the
need for config-builder to be able to generate genesis. Genesis requires the
VM as a dependency, which is not a dependency for loading or using the
configuration from many of the services.

## Building a Test Network

config-builder builds a single nodes configuration within a network or swarm of
nodes. This can be used to create a LIbra TestNet or to add a FullNetwork to an
existing Network. In addition, it enables generation of a mint / faucet client
capable of performing mint transactions / creating accounts.

## Generating a new TestNet

The only pre-requirements for generating a TestNet configuration is having a
list of IP Addresses and ports to host libra, a pre-agreed upon shared
secret, and a fixed ordering for each node in the network. Fullnode networks
can either be added to existing configs or generate completely new configs.

Each peer, I, can then generate their own configurations by:

    validator-config-builder \
        -a $PUBLIC_MULTIADDR_FOR_NODE_I \
        -b $PUBLIC_MULTIADDR_FOR_NODE_0 \
        -d /opt/libra/data \
        -i $I \
        -l $ANY_MULTIADDR_FOR_NODE_I \
        -n $TOTAL_NUMBER_OF_NODES \
        -o /opt/libra/etc \
        -s $SHARED_SECRET

As an example, this is the 2nd node (offset 1) in a set of 4:

    validator-config-builder \
        -a "/ip4/1.1.1.2/tcp/7000" \
        -b "/ip4/1.1.1.1/tcp/7000" \
        -d /opt/libra/data \
        -i 1 \
        -l "/ip4/0.0.0.0/tcp/7000" \
        -n 4 \
        -o /opt/libra/etc \
        -s 0123456789abcdef101112131415161718191a1b1c1d1e1f2021222324252627

To create a mint service's consensus peer config and key that connects to
this service:

    faucet-config-builder \
        -n 4 \
        -o /opt/libra/etc \
        -s 0123456789abcdef101112131415161718191a1b1c1d1e1f2021222324252627

Adding a full node network is similar to instantiating a validator config. The
difference is that if a config already exists at that path, the assumption is
to append the full node network, if the network isn't already defined.
Secondly, full nodes support public or permissioned networks. The former
requires no further configuration, whereas the latter needs a full node
specific seed, total number of full nodes, and index into the configuration
set. The total number includes the upstream peer which is indexed into the
first position (0).

    full-node-config-builder \
        -a $PUBLIC_MULTIADDR_FOR_NODE_I \
        -b $PUBLIC_MULTIADDR_FOR_NODE_0 \
        -d /opt/libra/data \
        -l $ANY_MULTIADDR_FOR_NODE_I \
        -n $TOTAL_NUMBER_OF_NODES \
        -o /opt/libra/etc \
        -s $SHARED_SECRET \
        [ -i $I -f $TOTAL_NUMBER_OF_FULL_NODES -c $FULL_NODE_SHARED_SECRET | -p ]

As an example a of adding 4 membered permissioned network connecting to the
node above:

    full-node-config-builder \
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

    full-node-config-builder \
        -a "/ip4/1.1.1.2/tcp/7100" \
        -b "/ip4/1.1.1.2/tcp/7100" \
        -d /opt/libra/fn/data \
        -l "/ip4/0.0.0.0/tcp/7100" \
        -n 4 \
        -o /opt/libra/fn/etc \
        -s 0123456789abcdef101112131415161718191a1b1c1d1e1f2021222324252627 \
        -p

## Internals

There are several configurations contained within Libra Configuration.

### Libra Node Configuration
The Libra Configuration contains several modules:

- AdmissionControlConfig - Where a Libra Node host their AC
- BaseConfig - Specifies the Nodes role and base directories
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
- NodeConfig - Host all configuration files for a Libra Node
- SafetyRulesConfig - Specifies the persistentcy strategy for Libra Safety
  Rules
- StateSyncConfig - Specifies parameters around state sycnhronization and the
  set of peers to that provide the data
- StorageConfig - Where the LibraDB is stored and its gRPC service endpoints
- VMConfig - Specifies the allowed publishing options and scripts that can be
  executed

### Client Configuration

- ConsensusPeersConfig - The clients source of truth for the set of Libra
  nodes (Validators and Full nodes derive this from genesis / blockchain).

### Shared Configuration

- TestConfig - Used during tests to hold account keys and temporary paths for
  the configurations that will automatically be cleaned up after the test.

## Testing
Configuration tests serve many purposes:

- Identifying when defaults have changed
- Ensuring that parsing / storing works consistently
- That default filename assumptions are maintained

Several of the defaults in the configurations, in particular paths and
addresses, have dependecies outside the Libra code base. These tests serve as
a reminder that there may be rammifications from breaking these that impact
production deployments.

The test configs currently live in `src/config/test_data`.

## TODO

- Add ability to turn off services that are optional (debug interface, AC
  gRPC)
- Eliminate ConsensusPeersConfig from ConsensusConfig and make it top level.
- Is the execution gRPC interface being deprecated? Cleanup configs.
- LoggerConfig should allow specifying the severity.
- Eliminate generate-keypair
- Generating public networks is broken
- Make SafetyRule use on disk by default and remove from config generator
