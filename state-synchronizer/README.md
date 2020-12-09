---
id: state_sync
title: State Synchronizer
custom_edit_url: https://github.com/diem/diem/edit/master/state-synchronizer/README.md
---
# State Synchronizer

The State Synchronizer is the component that helps nodes advance its local
ledger state by requesting chunks of transactions from peers that have more
up-to-date state. A node sends a chunk request to its peer, and the peers
responds with data that helps the node advance its local state.

## Overview

Refer to the [state synchronizer specifications](../specifications/state_sync)
for a high-level overview.

## Implementation Details

This crate contains an implemented instance of the specifications for
a DiemNet state synchronizer.

- `chunk_request` & `chunk_response`: definitions/implementation of state sync
DiemNet messages
- `coordinator`: the main runtime that processes messages from consensus
or DiemNet State Synchronizer network protocol
- `executor_proxy`: interface b/w state sync and storage/execution
- `request_manager`: actor that handles managing per-peer information
and sending chunk requests to advance local state based on per-peer info or network
health
- `synchronizer`: interface that manages state sync's interactions with consensus


## How is this module organized?
```
state-synchronizer
|- src
   |-- tests/                  # Unit/integration tests
   |-- chunk_request           # `GetChunkRequest` implementation
   |-- chunk_response          # `GetChunkResponse` implementation
   |-- coordinator             # coordinator-related code - see above for description
   |-- counters                # metrics
   |-- executor_proxy          # interface b/w state sync and storage/execution
   |-- logging                 # logging, for observability
   |-- network                 # interface with networking
   |-- request_manager         # request_manager code - see above
   |-- synchronizer            # synchronizer code - see above

```
