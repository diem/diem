---
id: network
title: Network
custom_edit_url: https://github.com/libra/libra/edit/master/network/README.md
---

# Network

The network component provides peer-to-peer communication primitives to other
components of a validator.

## Overview

The network component is specifically designed to facilitate the consensus and
shared mempool protocols. Currently, it provides these consumers with two
primary interfaces:
* RPC, for Remote Procedure Calls; and
* DirectSend, for fire-and-forget style message delivery to a single receiver.

The network component uses:
* TCP for reliable transport.
* [Noise](https://noiseprotocol.org/noise.html) for authentication and full
 end-to-end encryption.
* [Yamux](https://github.com/hashicorp/yamux/blob/master/spec.md) for
multiplexing substreams over a single connection.
* Push-style [gossip](https://en.wikipedia.org/wiki/Gossip_protocol) for peer
discovery.

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

* **NetworkProvider** &mdash; Exposes network API to clients. It forwards
requests from upstream clients to appropriate downstream components and sends
incoming RPC and DirectSend requests to appropriate upstream handlers.
* **Peer Manager** &mdash; Listens for incoming connections and dials other
peers on the network. It also notifies other components about new/lost
connection events and demultiplexes incoming substreams to appropriate protocol
handlers.
* **Connectivity Manager** &mdash; Ensures that we remain connected to a node
if and only if it is an eligible member of the network. Connectivity Manager
receives addresses of peers from the Discovery component and issues
dial/disconnect requests to the Peer Manager.
* **Discovery** &mdash; Uses push-style gossip for discovering new peers and
updates to addresses of existing peers. On every *tick*, it opens a new
substream with a randomly selected peer and sends its view of the network to
this peer. It informs the connectivity manager of any changes to the network
detected from inbound discovery messages.
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
    ├── benches                       # network benchmarks
    ├── memsocket                     # In-memory transport for tests
    ├── netcore
    │   └── src
    │       ├── multiplexing          # substream multiplexing over a transport
    │       ├── negotiate             # protocol negotiation
    │       └── transport             # composable transport API
    ├── noise                         # noise framework for authentication and encryption
    └── src
        ├── channel                    # mpsc channel wrapped in IntGauge
        ├── connectivity_manager       # component to ensure connectivity to peers
        ├── interface                  # generic network API
        ├── peer_manager               # component to dial/listen for connections
        ├── proto                      # protobuf definitions for network messages
        ├── protocols                  # message protocols
        │   ├── direct_send            # protocol for fire-and-forget style message delivery
        │   ├── discovery              # protocol for peer discovery and gossip
        │   ├── health_checker         # protocol for health probing
        │   └── rpc                    # protocol for remote procedure calls
        ├── sink                       # utilities over message sinks
        └── validator_network          # network API for consensus and mempool
