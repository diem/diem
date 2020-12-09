## API CHANGELOG

All noteable API changes may affect client (breaking, non-breaking changes) should be documented in this file.

Please add the API change in the following format:

```

## <date> (add "breaking change" to title following the date if a change will break client

- <describle one change of API>, please <PR link> if this log is added after the PR is merged.
- <describle another change of the API>

```

# 2020-12-07 Add a `X-Diem-Chain-Id`, `X-Diem-Ledger-Version` and `X-Diem-Ledger-TimestampUsec` to http response header

The added headers are same with JSON-RPC response json same name fields. They are added for client to know chain id and server's latest block version and timestamp without decoding body json.

* `X-Diem-Chain-Id`: chain id of server
* `X-Diem-Ledger-Version`: the latest ledger version when the request hit server.
* `X-Diem-Ledger-TimestampUsec`: the latest ledger version's timestamp in usec when the request hit server.

All values are string type.

# 2020-11-09 Add a `get_transactions_with_proofs` API

This allows to create verifying clients that do not need to blindly trust the fullnode they connect to.


## 2020-10-15 Add `accumulator_root_hash` to `get_metadata` method response

- [See PR #6536](https://github.com/diem/diem/pull/6536)


## 2020-10-05 Rename `upgradeevent` to `admintransaction` event
- Changed the name and structure for `upgradeevent`
- [See PR #6449](https://github.com/diem/diem/pull/6449)


## 2020-10-08 Decode transaction script as name, code, arguments and type_arguments

- Transaction [Script](docs/type_transaction.md#type-script) is fulfilled with known script name as type, code, arguments and type_arguments. Similar with on-chain transaction [Script](https://developers.diem.com/docs/rustdocs/diem_types/transaction/struct.Script.html) type.
- Renamed type `peer_to_peer_transaction` to `peer_to_peer_with_metadata`, which is consistent with stdlib transaction script name.
- Removed `mint_transaction`, it is never rendered because of a bug, and the script does not exist anymore.
- [See PR #6453](https://github.com/diem/diem/pull/6453)


## 2020-10-07 Add `diem_version` field to `get_metadata` response

- `diem_version` number of diem onchain version

See [doc](docs/type_metadata.md) for more details.


## [breaking] 2020-10-07 Rename unknown script type from `unknown_transaction` to `unknown`

- `unknown_transaction` may cause user think the transaction is invalid. Change to `unknown` which is align with other unknown types.

This is breaking change if a client used `unknown_transaction` to handle result.

See [doc](docs/type_transaction.md#type-script) for more details.


## 2020-10-06 Add `script_hash_allow_list` and `module_publishing_allowed` fields to `get_metadata` method response

- `script_hash_allow_list` returns list of allowed scripts hash.
- `module_publishing_allowed` returns bool value indicates whether publishing customized scripts are allowed

See [doc](docs/type_metadata.md) for more details.


## 2020-10-05 Add `created_address` and `role_id` fields to `CreateAccount` event

- `created_address` is the address created account.
- `role_id` is the role id of the created account.

## 2020-09-30 Add `CreateAccount` event

- New event data type createaccount.

## [breaking] 2020-09-09

- In `KeptVMStatus`, `VerificationError` and `DeserializationError` were merged into `MiscellaneousError`
- This merger was reflected in `VMStatusView`
- [See PR #5798](https://github.com/diem/diem/pull/5798)

## 2020-09-02 Add `address` to `get_account` response.
- Adding address field to get_account response.

## 2020-08-25 Add `BaseUrlRotation` and `ComplianceKeyRotation` events
- Added two new event types, one is emitted when the public key used for dual
  attestation on chain is rotated, and the other is emitted when the base url
  used for dual attestation off-chain communication is changed.

## 2020-08-11 Added chain_id to get_metadata

- added "chain_id" field to `get_metadata` response so it is available outside
  of the root JSON-RPC Response object

## [breaking] 2020-08-10 Adding missing "type" tag for AccounRole and VMstatus.

- adding "type" tag for values in "vm_status" field returned in transction object in method get_transcations, get_account_transcation etc.
- adding "type" tag for values in "role" field returned in account object in method get_account.

## 2020-09-24 Add human-readable explanations to Move abort statuses
- Adds an optional human-readable category, reason, and explanations for each
  Move abort code in the `MoveAbortExplanationView`.

## Before 2020-08-05

Please refer to [JSON-RPC SPEC before 2020-08-05](https://github.com/diem/diem/blob/888e6cd688a8c9b5805978ab509acdc3c35025ab/json-rpc/json-rpc-spec.md) document for the API spec snapshot.
