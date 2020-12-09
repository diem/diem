---
id: network
title: Network
custom_edit_url: https://github.com/diem/diem/edit/master/network/README.md
---


The network component provides peer-to-peer communication primitives to other
components of a validator.

## Overview

For more detailed info, see the [Network Specification](../specifications/network/README.md).

The network component is specifically designed to facilitate the consensus and
shared mempool protocols. Currently, it provides these consumers with two
primary interfaces:
* RPC, for Remote Procedure Calls; and
* DirectSend, for fire-and-forget style message delivery to a single receiver.

The network component uses:
* TCP for reliable transport.
* [Noise](https://noiseprotocol.org/noise.html) for authentication and full
 end-to-end encryption.
* On-chain addresses for discovery, with a seed peers file for initial discovery.

Each new substream is assigned a *protocol* supported by both the sender and
the receiver. Each RPC and DirectSend type corresponds to one such protocol.

Only eligible members are allowed to join the inter-validator network. Their
identity and public key information is provided by the consensus
component at initialization and on validator set reconfiguration. A new
validator also needs the network addresses of a few *seed* peers to help it
bootstrap connectivity to the network. The seed peers first authenticate the
joining validator as an eligible member and then share their network state
with it.

Each member of the network maintains a full membership view and connects
directly to any validator it needs to communicate with. A validator that cannot
be connected to directly is assumed to fall in the quota of Byzantine faults
tolerated by the system.

Validator health information, determined using periodic liveness probes, is not
shared between validators; instead, each validator directly monitors its peers
for liveness.

This approach should scale up to a few hundred validators before requiring
partial membership views, sophisticated failure detectors, or network overlays.

## Implementation Details

### System Architecture

    +--------------------+----------------------+
    |      Consensus     |        Mempool       |
    +--------------------+----------------------+
    |               Peer Manager                |
    +-------------------------------------------+
    |              NetworkProvider(s)           |
    +--------------------+----------------------+
    |        RPC(s)      |      DirectSend(s)   |
    +--------------------+----------------------+
    |                  Peer(s)                  |
    +-------------------------------------------+

The network component is implemented in the
[Actor](https://en.wikipedia.org/wiki/Actor_model) programming model &mdash;
i.e., it uses message-passing to communicate between different subcomponents
running as independent "tasks." The [tokio](https://tokio.rs/) framework is
used as the task runtime. The different subcomponents in the network component
are:

* **NetworkProvider** &mdash; An application level client interface to the network API.
It forwards requests from upstream clients to appropriate downstream components and sends
incoming RPC and DirectSend requests to appropriate upstream handlers.

* **Peer Manager** &mdash; Listens for incoming connections, and dials outbound
connections to other peers.  Demultiplexes and forwards messages to appropriate
protocol handlers.  Additionally, notifies upstream components of new or closed
connections.  Optionally can be connected to ConnectivityManager for a network with
Discovery.

* **Connectivity Manager** &mdash; Establishes connections to known peers found via
Discovery.  Notifies Peer Manager to make outbound dials, or disconnects based
on updates to known peers via Discovery updates.  This is only needed on a network
that uses Discovery e.g. the Validator network.

* **OnChain Discovery** &mdash; Discovers via OnChain configuration the set of peers
to connect to.  In the case of the Validator network, this is the ValidatorSet.
Notifies ConnectivityManager of updates to the known peer set.

* **Health Checker** &mdash; Performs periodic liveness probes to ensure the
health of a peer/connection. It resets the connection with the peer if a
configurable number of probes fail in succession. Probes currently fail on a
configurable static timeout.

* **Direct Send** &mdash; Allows sending/receiving messages to/from remote
peers. It notifies upstream handlers of inbound messages.

* **RPC** &mdash; Allows sending/receiving RPCs to/from other peers. It notifies
upstream handlers about inbound RPCs. The upstream handler is passed a channel
through which can send a serialized response to the caller.

In addition to the subcomponents described above, the network component
consists of utilities to perform encryption, transport multiplexing, protocol
negotiation, etc.

## How is this module organized?

    network
    ├── benches                        # network benchmarks
    ├── builder                        # Builds a network from a NetworkConfig
    ├── memsocket                      # In-memory transport for tests
    ├── netcore
    │   └── src
    │       └── transport              # composable transport API
    ├── network-address                # network addresses and encryption
    |── simple-onchain-discovery       # protocol for peer discovery
    └── src
        ├── connectivity_manager       # component to ensure connectivity to peers
        ├── interface                  # generic network API
        ├── noise                      # noise integration
        ├── peer_manager               # component to dial/listen for connections
        ├── protocols                  # message protocols
        │   ├── direct_send            # protocol for fire-and-forget style message delivery
        │   ├── health_checker         # protocol for health probing
        │   ├── network                # components for interaction with applications
        │   ├── rpc                    # protocol for remote procedure calls
        │   └── wire                   # protocol for DiemNet handshakes and messaging
        └── testutils                  # utilities for testing
