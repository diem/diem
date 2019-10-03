---
id: admission-control
title: Admission Control
custom_edit_url: https://github.com/libra/libra/edit/master/admission_control/README.md
---
# Admission Control

Admission Control (AC) is the public API endpoint for Libra and it takes public gRPC requests from clients.

## Overview
Admission Control (AC) serves two types of requests from clients:
1. SubmitTransaction - To submit a transaction to the associated validator.
2. UpdateToLatestLedger - To query storage, e.g., account state, transaction log, proofs, etc.

## Implementation Details
Admission Control (AC) implements two public APIs:
1. SubmitTransaction(SubmitTransactionRequest)
    * Multiple validations will be performed against the request:
	   * The Transaction signature is checked first. If this check fails, AdmissionControlStatus::Rejected is returned to client.
	   * The Transaction is then validated by vm_validator. If this fails, the corresponding VMStatus is returned to the client.
	* Once the transaction passes all validations, AC queries the sender's account balance and the latest sequence number from storage and sends them to Mempool along with the client request.
    * If Mempool returns MempoolAddTransactionStatus::Valid, AdmissionControlStatus::Accepted is returned to the client indicating successful submission. Otherwise, corresponding AdmissionControlStatus is returned to the client.
2. UpdateToLatestLedger(UpdateToLatestLedgerRequest). No extra processing is performed in AC.
* The request is directly passed to storage for query.

## How is this module organized?
```
    .
    ├── README.md
    ├── admission_control_proto
    │   └── src
    │       └── proto                           # Protobuf definition files
    └── admission_control_service
        └── src                                 # gRPC service source files
            ├── admission_control_service.rs    # gRPC service and main logic
            ├── main.rs                         # Main entry to run AC as a binary
            └── unit_tests                      # Tests
```

## This module interacts with:
The Mempool component, to submit transactions from clients.
The Storage component, to query validator storage.
