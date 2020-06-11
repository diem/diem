---
id: state-sync
title: State Synchronizer
custom_edit_url: https://github.com/libra/libra/edit/master/state-synchronizer/README.md
---
# State Synchronizer

State synchronizer is used by full nodes and validators to synchronize to the blockchain.

## How is this module organized?

```
state-synchronizer/src
├── chunk_request.rs  # request code
├── chunk_response.rs # response code
├── coordinator.rs    # main loop
├── counter.rs        # counters we update for metrics
├── executor_proxy.rs # communication with executor and storage
├── lib.rs            # internal state of state-sync
├── network.rs        # communication with network
├── peer_manager.rs   # peer management and scoring
└── synchronizer.rs   # bootstrapping of state-sync and client handle
```
