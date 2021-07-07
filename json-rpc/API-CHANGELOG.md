## API CHANGELOG

All noteable API changes may affect client (breaking, non-breaking changes) should be documented in this file.

Please add the API change in the following format:

```

## <date> (add "breaking change" to title following the date if a change will break client

- <describe one change of API>, please <PR link> if this log is added after the PR is merged.
- <describe another change of the API>

```

## 2021-07-07 Add `get_event_by_version_with_proof` API

This new API allows light clients to request an event at or below a version.
Concretely, light clients can now fully support `get_metadata` by using this API.

## 2021-06-08 Add `get_account_transactions_with_proofs` API

This new API allows verifying client users to get transactions under an account
along with the corresponding proofs needed to verify the response.

## 2021-06-07 `get_account_transactions` on a non-existent account returns empty list istead of error

This change makes `get_account_transactions` consistent with `get_account_transaction`,
which returns nothing on a non-existent account and removes an extra internal DB
call that is no longer needed.

## 2021-06-01 Add `get_accumulator_consistency_proof` API

This API allows light clients to build their initial transaction accumulator summary
and verifiably extend their accumulator summary to newer ledger states.

## 2021-05-25 Add `TreasuryComplianceRole`

TreasuryComplianceRole has been created and has a field `vasp_domain_events_key` that stores the event key of vasp domain events.

## 2021-05-24 Add `vasp_domains` field to `ParentVASPRole`

ParentVASPRole has a new field `vasp_domains` that stores the domain names of the parent VASP.

## 2021-05-07 Add support for secondary signers in `TransactionDataView`

With the introduction of multi-agent transactions, multiple accounts are now able to
participate in and sign a transaction. Therefore, we have added four new fields,
`secondary_signers`, `secondary_signature_schemes`, `secondary_signatures`,
and `secondary_public_keys`, to support this new feature.

## 2021-05-12 Add `include_events` parameter and `serialized_events` field to `get_transactions_with_proofs` method

- Add an `include_events` parameter to the `get_transactions_with_proofs` method,
  which optionally includes the `serialized_events` field in the response. This
  field is a BCS-serialized list of each transaction's list of generated events.
- Add a field `first_transaction_version` to the `get_transactions_with_proofs`
  method response, which contains the version of the first transaction in the
  response or `null` if there are no transactions.

## 2021-04-21 Add `bytes` field to `unknown` event

When the event data can't be parsed, we output `unknown` type event data with raw BCS event data bytes, so that client can unwrap event details by parsing the BCS bytes.

## 2021-03-25 Add `metadata` field to preburns in the `preburn_queues` for designated dealers

This adds an additional `metadata` field coupled with each preburn request held
in the `preburn_queues` for a designated dealer. This also adds a new `PreburnWithMetadataView`.

## 2021-03-25 Add `arguments_bcs` in `ScriptView` for hex-encoded BCS bytes

The `ScriptFunction` transaction payload accepts BCS-encoded bytes as arguments
while the traditional `Script` payload type still accepts arguments wrapped in
`TransactionArgument`. To differentiate them in `ScriptView` returned to users,
a `arguments_bcs` field is added which will display HEX-encoded BCS bytes for
the `ScriptFunction` payload while the arguments in the `Script` payload is
still displayed through `arguments`.

## 2021-03-18 Add optional parameter `version` to `get_account` method

- Get account data by version.
- A new field `version` is added to [Account](docs/type_account.md) in the `get_account` method response.

## 2021-03-16 Add support for script functions in `TransactionDataView` and `ScriptView`

Update `ScriptView` to support script functions. This adds three new nullable
fields to the `ScriptView` data type; `module_address`, `module_name`, and
`function_name`. These fields are set to non-null values for script functions,
and when the `ScriptView` `type` field is set to `"script_function"`.  These
three new fields for script functions together represent a fully-qualified call
into a Move module of the form `<module_address>::<module_name>::<function_name>`.

Additionally, within the `UserTransaction` variant of the `TransactionDataView`
for script functions, the `code` field is empty as no code is sent with the
transaction, and the script bytes remain the BCS serialized bytes of the script
being sent.

## 2021-03-09 Add `preburn_queues` to `AccountRoleView` for designated dealers

Designated dealers will now be able to have multiple preburn requests in-flight
at the same time. This means that the `preburn` resource published under their
account will be removed, and replaced with a `preburn_queue` resource the next
time they send a preburn request. The view for a Designated Dealer has been
updated to represent this new preburn queue and the outstanding preburn
requests in it in the new field `preburn_queues`. This change is non-breaking
as the values in the `preburn_balances` field are kept consistent with the sum
of all outstanding burn requests in the preburn queue.

## 2020-01-08 Removing known_version from verifying APIs

Since the servers are pruning old ledger info states, it is easy for any call with a given known_version value to fail, unless that version is the last one of an epoch, since then it is preserved to provide epoch change proofs.
To use the verifying API, people are expected to do batch calls with a `get_state_proof` call to make sure they get a matching response from the server.

## 2020-12-22 Add `get_events_with_proof` APIs

This allows to verify events since transactions and their proofs are not covering all events.


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
