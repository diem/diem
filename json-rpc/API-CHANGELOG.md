## API CHANGELOG

All noteable API changes may affect client (breaking, non-breaking changes) should be documented in this file.

Please add the API change in the following format:

```

## <date> (add "breaking change" to title following the date if a change will break client

- <describle one change of API>, please <PR link> if this log is added after the PR is merged.
- <describle another change of the API>

```

```
## [breaking] 2020-08-10 Adding missing "type" tag for AccounRole and VMstatus.

- adding "type" tag for values in "vm_status" field returned in transction object in method get_transcations, get_account_transcation etc.
- adding "type" tag for values in "role" field returned in account object in method get_account.

```

## Before 2020-08-05

Please refer to [JSON-RPC SPEC before 2020-08-05](https://github.com/libra/libra/blob/888e6cd688a8c9b5805978ab509acdc3c35025ab/json-rpc/json-rpc-spec.md) document for the API spec snapshot.
