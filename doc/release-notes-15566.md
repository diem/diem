Miscellaneous CLI Changes
-------------------------
- The `testnet` field in `bitcoin-cli -getinfo` has been renamed to `chain` and now returns the current network name as defined in BIP70 (main, test, regtest).