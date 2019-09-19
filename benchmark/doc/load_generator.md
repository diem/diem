
# Load Generation

## Definition of Load

In benchmarking experiments, we need a module to generate transactions, or more generally, generate workloads. In the setting where Libra is considered as a database, if we think `SubmitTransactionRequest` as write requests, it is natural to also consider read requests, e.g., `UpdateToLatestLedgerRequest`. We have the following enum to unify both read and write requests:

```
pub enum Request {
    // Both write and read requests are protobuf struct.
    WriteRequest(SubmitTransactionRequest),
    ReadRequest(ProtoUpdateToLatestLedgerRequest),
}
```

## Load Generator Interface

Any struct that implements the following two methods can be used in benchmark experiment for load generation. The two methods simply generate accounts, and requests that come from the activity between generated accounts.

```
/// This interface specifies the requirements to generate customized read/write requests.
/// Required methods are expected to be called in following specified order within each round:
pub trait LoadGenerator {
    /// 1. Generate arbitrary number of new accounts.
    fn gen_accounts(&mut self, num_accounts: u64, round_index: u64) -> Vec<AccountData>;
    /// 2. Generate arbitrary read/write requests from a set of accounts.
    ///    Return generated requests and indices to the sender accounts.
    fn gen_requests(&self, accounts: &mut [AccountData], round_index: u64) -> (Vec<Request>, Vec<usize>);
}
```
Current code in [load_generator.rs](https://github.com/libra/libra/blob/master/benchmark/src/load_generator.rs) does not fully match this API but is planed to evolved according to the description here ([Issue 807](https://github.com/libra/libra/issues/807)). Planed changes include getting rid of `gen_setup_requests()` and introducing `round_index` in both `gen_accounts()` and `gen_requests()`.

Currently we provide two transaction generators:
1. Ring transaction generator: given account (A1, A2, A3, ..., AN), returns a vector of circular transactions like [A1→A2, A2→A3, A3→A4, ..., AN→A1]. So N accounts means N transactions.
2. Pairwise transaction generator: given account (A1, A2, A3, ..., AN), returns a vector of transactions like [A1→A1, A1→A2, ..., A1→AN, A2→A1, A2→A2, ... A2→AN, ..., AN→AN]. Note self-to-self transfer is included. This means N accounts can generate N^2 transactions.


## Workflow with Benchmarker

The current implementation has not considered `round_index` in both account and transaction generation, because existing transaction generators only generate one round of transactions and then repeat the same pattern for several rounds. However, as mentioned in [Issue 807](https://github.com/libra/libra/issues/807), we plan to use load generator with Benchmarker in the following way:

```
/// Assume we already have Benchmarker bm, generator impl LoadGenerator.
let faucet_account = bm.load_faucet_account();
let accounts = [faucet_account];
for r in 0...num_rounds {
    // Incrementally generate new accounts in new round.
    let new_accounts = generator.gen_accounts(num_accounts, r);
    bm.register_accounts(new_accounts);
    accounts.append(new_accounts);

    // Generate transactions in particular round.
    let (requests, sender_indices) = generator.gen_load_requests(accounts, r);
    bm.submit_requests_and_wait_txns_committed(accounts[sender_indices], requests);
}
```

Initially we load faucet account from one validator. Then in each round, we extend our account records with several new accounts. Generator takes in all accounts and generates transactions for this round, along with the transaction senders' identity. Then Benchmarker can submit the requests (which can be both transactions or read requests) and wait only on senders' sequence numbers reaching expected values.

The most interesting future usage of load generator will be in playing real world transactions from Bitcoin or Ethereum on Libra. Within the previous running framework, we need a customized generator. It should be customized to understand the transaction dependency, for example, parsing the ordered transactions or blocks from a particular dataset, e.g. [EthereumEtl](https://github.com/blockchain-etl/ethereum-etl). The generator then treats each block as a round, as transactions inside a block can assume to be mutually independent. A cleverer way would be merging mutually independent blocks first.
At the beginning of each round, we first generate/mint accounts newly appeared on the fly. Then generate generates new transactions, along with their sender's info, from the entire account list. With generated transactions and knowing who are the senders, Benchmarker will start to play the transactions.
