# Protecting against Denial of Service Attack

To advertise the benchmarking library, this section describe how we test Libra's DoS resistance using [Benchmarker](README.md).
We first assume there is no per-IP filtering protecting the system but will discuss what is needed when such mechanism exists.

## DoS Attack with Transaction Requests
First, we need to admit that launching denial-of-service attack to Libra with transaction requests is certainly not a trivial task. Even if you can generate valid and signed transactions on your own very efficiently, mempool will not let these transactions pass through because it limits the number of transactions on a per account basis. Mempool has a capacity to hold a large number of not-ready transactions and garbage collect expired transactions. Consensus also processes mempool's transactions at its own speed.
However, it is still possible to design attack that exploits mempool's capacity with sufficiently large amount of valid and long-lifetime transactions.

## DoS Attack with `UpdateToLatestLedgerRequest`s
However, we manage to find a bottleneck that breaks down Libra with `UpdateToLatestLedgerRequest`. In the preparation step, we create around 1000 accounts and mint them in multiple rounds, to ensure that in each round the faucet account only have less than 100 mint transactions. Then we craft a template `GetTransactions` request which asks for 1000 committed transactions from the blockchain network. Notice that unlike transaction request, we don't need to do any signing; so this template request can be directly copied and used by any number of clients. Initially 32 clients in Benchmarker sends 10 exactly the same requests at 1 request per second. Then we gradually increase the number of requests as well as the speed of request.
The core difference is that submitting a transaction (when there is no batch API) has an upper cost. The cost of big `UpdateToLatestLedgerRequest`, these querying large amount of transactions, has a higher upper cost.

### DoS Local Libra Swarm

First we tested this attack locally. AC is responding our requests in a rate of around 125 responses per second, and the size of responses is in the magnitude of tens of megabytes. But more interesting is that local Libra swarm's CPU usage has reached nearly 100%, and it stays at such a high utilization. As Benchmarker is maliciously sending `GetTransaction` requests to `libra-swarm`, I tried to start a normal client but it fails to connect to `libra-swarm`. At surface the reason is that the client cannot load faucet account's state from validator, as gRPC request timed out. The root reason, however, is that particular validator is not able to handle any incoming request because it is overwhelmed by the `GetTransaction` requests.

### DoS Libra Running as AWS Clusters

One may argue it is relatively easy to DoS locally running Libra because system resources are not well isolated. So we replayed the same attack on a 4-node Libra network running on AWS. The result is almost the same: all four validators have reached 90% CPU utilization.

We traced down the reason by running [perf](https://perf.wiki.kernel.org/index.php/Main_Page) on the validators. The reported metrics here show that high CPU utilization comes from the deserialization phase within the `get_transaction()` procedure. Particularly, it is when deserializing the public key in `crypto/nextgen_crypto/src/ed25519.rs`:

```
let compressed = curve25519_dalek::edwards::CompressedEdwardsY(bits);
let point = compressed
    .decompress()
    .ok_or(CryptoMaterialError::DeserializationError)?;
```

We further refined the DoS attack and eventually makes validators reach and constantly stay at near 100% CPU utilization using only 25% of CPU on the Benchmarker side.
In order to save its own resources, the straightforward action for attacker is to reduce the number of requests but still keep validators busy enough. In our implementation, we reset number of requests and submission rate to its initial values when request rate reaches a predefined threshold value (8K requests per second). We also observe such attack takes effective very quickly. It only takes 4 mins for the attacker to take up all 4 nodes' CPU resources.

One critical requirement to implement this attack in real world is to have a large number of IP addresses, since in production system usually rate limit requests by IP address. For estimation purpose, if we assume the attacking speed is 8K req per second, then launching the attack will takes more than 10K IP addresses. But notice that attackers don't need to always keep their submission rate at 8K, and they even don't need to reach 8K.
