## API CHANGELOG

All noteable API changes may affect client (breaking, non-breaking changes) should be documented in this file.

Please add the API change in the following format:

```

## <date> (add "breaking change" to title following the date if a change will break client

- <describle one change of API>, please <PR link> if this log is added after the PR is merged.
- <describle another change of the API>

```
## [breaking] 2020-09-09

- In `KeptVMStatus`, `VerificationError` and `DeserializationError` were merged into `MiscellaneousError`
- This merger was reflected in `VMStatusView`
- [See PR #5798](https://github.com/libra/libra/pull/5798)

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

## Before 2020-08-05

Please refer to [JSON-RPC SPEC before 2020-08-05](https://github.com/libra/libra/blob/888e6cd688a8c9b5805978ab509acdc3c35025ab/json-rpc/json-rpc-spec.md) document for the API spec snapshot.
