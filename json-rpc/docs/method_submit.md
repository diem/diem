
## Method submit

**Description**

Submit a signed transaction to a full node.


### Parameters

| Name  | Type     | Description                                                                                          |
|-------|----------|------------------------------------------------------------------------------------------------------|
| data  | string   | Signed transaction data - hex-encoded bytes of [BCS][1] serialized Diem [SignedTransaction][3] type.|

Steps to create "data" parameters:

1. Create [RawTransaction][2]
2. Create signing message:
   1. [SHA3](https://en.wikipedia.org/wiki/SHA-3) hash bytes of string "DIEM::RawTransaction".
   2. [RawTransaction][2] [BCS][1] serialized bytes.
   3. Concat above 2 bytes.
3. Sign the message with [Ed25519](https://ed25519.cr.yp.to/) and your account private key.
4. Create [SignedTransaction][3]
5. Serialize [SignedTransaction][3] into bytes, and then hex-encode into string as Signed transaction data parameter.

See more related at [Crypto Spec](../../specifications/crypto/README.md)

### Returns

Null - on success

Note:
* Although submit returns errors immediately for the invalid transaction submitted, but success submit does not mean the transaction will be executed successfully.
* Client should call [get_account_transaction](method_get_account_transaction.md) with the sender account address and the RawTransaction sequence_number to find out whether transaction is executed.
* After transaction is submitted, but not executed yet, calling [get_account_transaction](method_get_account_transaction.md) will return null.
* If [get_account_transaction](method_get_account_transaction.md) returns a Transaction, client should validate [Transaction#signature](type_transaction.md#user) == the SignedTransaction signature as it is possible there is another Transaction submitted with same account sequence number.
* After validated the Transaction is the submitted SignedTransaction, client should confirm the transaction is executed successfully by checking [Transaction#vm_status] == "executed"; any other vm_status means execution failed. The vm_status may contain some information for client to understand what's going wrong.
* There is no partial execution in Diem, hence the transaction either has full effect or has no effect at all.
* It is possible a Transaction may not executed after submitted successfully, hence you can't find it by [get_account_transaction](method_get_account_transaction.md) method. To avoid endless waiting, client should setup a reasonable Transaction expiration timestamp (RawTransaction#expiration_timestamp_secs), client may keep trying [get_account_transaction](method_get_account_transaction.md) and check diem_ledger_timestampusec in the response with the Transaction expiration timestamp. Transaction won't be executed if it's expiration timestamp is passed, hence client can safely re-construct the Transaction with new expiration timestamp and submit again.

### Errors

Errors during the transaction are indicated by different error codes:

| Code   | Description                                                        |
|--------|--------------------------------------------------------------------|
| -32000 | Default server error                                               |
| -32001 | VM validation error                                                |
| -32002 | VM verification error                                              |
| -32003 | VM invariant violation error                                       |
| -32004 | VM deserialization error                                           |
| -32005 | VM execution error                                                 |
| -32006 | VM unknown error                                                   |
| -32007 | Mempool error: invalid sequence number                             |
| -32008 | Mempool is full error                                              |
| -32009 | Mempool error: account reached max capacity per account            |
| -32010 | Mempool error: invalid update (only gas price increase is allowed) |
| -32011 | Mempool error: transaction did not pass VM validation              |
| -32012 | Unknown error                                                      |

More information might be available in the “message” field, but this is not guaranteed.
For VM and Mempool errors may include a "data" object contains more detail information.


### Example


```
// Request: submits a transaction whose hex-encoded BCS byte representation is in params
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"submit","params":["1668F6BE25668C1A17CD8CAF6B8D2F25370000000000000001E101A11CEB0B010000000701000202020403061004160205181D0735610896011000000001010000020001000003020301010004010300010501060C0108000506080005030A020A020005060C05030A020A020109000C4C696272614163636F756E741257697468647261774361706162696C6974791B657874726163745F77697468647261775F6361706162696C697479087061795F66726F6D1B726573746F72655F77697468647261775F6361706162696C69747900000000000000000000000000000001010104010C0B0011000C050E050A010A020B030B0438000B05110202010700000000000000000000000000000001034C4252034C4252000403262E691EC8C7E3E23470D8C3EE26E1A70140420F00000000000400040040420F00000000000000000000000000034C425200E8764817000000020020F549A91FB9989883FB4D38B463308F3EA82074FB39EA74DAE61F62E11BF55D25405CD26F114183C44874DD3F861E0AD24B8E5D8B8C1CAA1B79C7E641C664AE3FD645E4310237A0DC046046DEFBE27C4F15CAAB55A76BBAC15E92B444431232DE0C"],"id": 1}' https://testnet.diem.com/v1

// Response, for successful transaction submission
{
  "id":1,
  "jsonrpc":"2.0",
  "diem_chain_id":2,
  "diem_ledger_timestampusec":1596736351198722,
  "diem_ledger_version":3475232,
  "result":null
}
```

[1]: https://docs.rs/bcs/ "BCS"
[2]: https://developers.diem.com/docs/rustdocs/diem_types/transaction/struct.RawTransaction.html "RawTransaction"
[3]: https://developers.diem.com/docs/rustdocs/diem_types/transaction/struct.SignedTransaction.html "SignedTransaction"
