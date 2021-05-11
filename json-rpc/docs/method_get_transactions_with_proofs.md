## Method get_transactions_with_proofs

**Description**

Get the transactions on the blockchain along with the proofs necessary to verify said transactions.

### Parameters

| Name           | Type           | Description                                                           |
|----------------|----------------|-----------------------------------------------------------------------|
| start_version  | unsigned int64 | Start on this transaction version for this query                      |
| limit          | unsigned int64 | Limit the number of transactions returned, the max value is 1000      |
| include_events | boolean        | When set to true, also include serialized events for each transaction |

### Returns

If include_events is false, the serialized_events field will not be included.

| Name                      | Type                   | Description                                                                                                 |
|---------------------------|------------------------|-------------------------------------------------------------------------------------------------------------|
| serialized_transactions   | List<string>           | An array of hex encoded strings with the raw bytes of the returned `Transaction`                            |
| serialized_events         | Option<string>         | An optional BCS-serialized list of each transaction's list of events or `null` if `include_events` is false |
| first_transaction_version | Option<unsigned int64> | If there are any transactions, then the version of the first transaction, otherwise `null`                  |
| proofs                    | TransactionsProofs     | The proofs, see below.                                                                                      |


The proofs:

| Name                                   | Type   | Description                                                                                                                                              |
|----------------------------------------|--------|----------------------------------------------------------------------------------------------------------------------------------------------------------|
| ledger_info_to_transaction_infos_proof | string | An hex encoded string of raw bytes of a `Vec<AccumulatorRangeProof<TransactionAccumulatorHasher>>` that contains the proofs of the returned transactions |
| transaction_infos                      | string | An hex encoded string of raw bytes of a `Vec<TransactionInfo>` that corresponds to returned transcations                                                 |

Note:
 - all raw bytes encoded strings are containing BCS encoded data.
 - you need the `ledger_info` at the time of the request in order to have the correct `accumulator_hash` to verify the `ledger_info_to_transaction_infos_proof` produced at that time, so you should do a batched call to `get_state_proof` whenever you call `get_transactions_with_proofs`. See the integration test for an example.

In order to see an example of how to verify the proofs, please refer to the [integration tests](json-rpc/tests/integration_test.rs).
