## JSON-RPC Client Implementation Guide

### Overview

To implement a client connecting to Diem JSON-RPC APIs, you need consider the followings:

* [JSON-RPC client](#json-rpc-client): talks to Diem JSON-RPC server.
* [Testnet](#testnet): connect to testnet to do integration tests and confirm your client works as expected.
* [Query Blockchain](#query-blockchain): read data from Diem blockchain.
* [Submit Transaction](#submit-transaction): write data to Diem blockchain.
* [Error handling](#error-handling): handle errors.

### JSON-RPC client

Any JSON-RPC 2.0 client should be able to work with Diem JSON-RPC APIs.
Diem JSON-RPC APIs extend to JSON-RPC 2.0 Spec for specific use case, check [Diem Extensions](./../json-rpc-spec.md#diem-extensions) for details, we will discuss more about them in [Error Handling](#error-handling) section.

### Testnet

A simplest way to validate your client works is connecting it to Testnet(https://testnet.diem.com/v1).
For some query blockchain methods like [get_currencies](method_get_currencies.md) or [get_metadata](method_get_metadata.md), you don't need anything else other than a HTTP client to get back response from server.
Try out [get_currencies example](method_get_currencies.md#example) on Testnet, and this can be the first query blockchain API you implement for your client.

When you need a test [submit transaction method](method_submit.md), like peer to peer transfer coins, you will need accounts created for both sender and receiver. Technically it is a transaction (with creating account script) that needs to be submitted and executed, but creating account script is permitted to special accounts, and Testnet does not publish these accounts' private key, thus you can't do it on your own.

Instead, we created a service named `Faucet` for anyone who wants to create an account and mint coins on Testnet.
Please follow [Testnet Faucet Service](service_testnet_faucet.md) to implement mint coins for testing accounts, then you are ready to test submit a peer to peer transfer transaction.

### Query Blockchain

All the methods prefixed with `get_` listed at [here](./../json-rpc-spec.md#overview) are designed for querying Diem blockchain data.

You may start with implementing [get_currencies](method_get_currencies.md), which is the simplest API that does not require any arguments and always responds to the same result on Testnet.

When you implement [get_account](method_get_account.md), you can use the following 3 static account address to test the API against with Testnet:

| account name            | address hex-encoded string       | description                                                                                                                |
|-------------------------|----------------------------------|----------------------------------------------------------------------------------------------------------------------------|
| root account address    | 0000000000000000000000000A550C18 | A special root account, stores important global resource information like all currencies info                              |
| core code address       | 00000000000000000000000000000001 | A special code account, we will need it for submit transaction, it stores currency code type info                          |
| designed dealer address | 000000000000000000000000000000DD | A special account for minting coins on Testnet, checkout [Testnet Faucet Service](service_testnet_faucet.md) for more info |

As the above account addresses are static on Testnet, it is convenient for you to test against them for [get_account](method_get_account.md) method.
However, if you implemented [Testnet Faucet Service](service_testnet_faucet.md), you can create your own testing account for testing on Testnet.

Similarly, we can test our [get_account_transaction](method_get_account_transaction.md) implementation with `root account address`.

> We need call [get_account](method_get_account.md) and [get_account_transaction](method_get_account_transaction.md) when we implement and test [Submit Transaction](#submit-transaction) method.
> So you should at least implement and confirm these two methods are working as expected.

### Submit Transaction

To implement submitting a transaction, you may follow the following steps:

1. [Create local account](#create-local-account): it includes an address, [Ed25519](https://ed25519.cr.yp.to/) generate private key and public key, an authentication key that is generated from public key.
2. [Create and sign transaction](#create-and-sign-transaction)
3. [Submit transaction](#submit-transaction)
4. [Wait for transaction executed and validate result](#wait-for-transaction-executed-and-validate-result): the execution can fail after you submitted successfully.

The following diagram shows the sequence of submit and wait for a peer to peer transaction executed successfully:

![Submit and wait for transaction executed successfully](images/submit_wait_transaction.png)

#### Create local account

A local account holds secrets of an onchain account: the private key.
Maintaining the local account or keeping the secure of private key is out of a Diem client's scope. In this guide, we use [Diem Swiss Knife][6] to generate local account keys:

``` shell
# generate test keypair
cargo run -p swiss-knife -- generate-test-ed25519-keypair

{
  "error_message": "",
  "data": {
    "diem_account_address": "a74fd7c46952c497e75afb0a7932586d",
    "diem_auth_key": "459c77a38803bd53f3adee52703810e3a74fd7c46952c497e75afb0a7932586d",
    "private_key": "cd9a2c90296a210249128ae3c908611637b2e00efd4986670e252abf3fabd1a9",
    "public_key": "447fc3be296803c2303951c7816624c7566730a5cc6860a4a1bd3c04731569f5"
  }
}

```

> To run this by yourself, clone https://github.com/diem/diem.git, and run `./scripts/dev_setup.sh` to setup dev env.
> You can run the command in the above example at the root directory of diem codebase.


#### Create and sign transaction

Now we have a local account address and keys, we can start to prepare a transaction.
In this guide we use peer to peer transfer as an example, others will be similar except some scripts can only be submitted by specific accounts.

There are several development tools available for you:

1. [Transaction Builder Generator][2]: this is actively in development, currently supports C++, Java, Python and Rust ([latest language supports][10]).
2. swiss-knife: check out [Swiss Knife generate raw transaction and sign transaction][8]; when we don't have transaction builder generator in the language you want to develop the client, you can wrap the swiss-knife [release binary][9] for creating and signing transaction.
3. [C-binding][11]: if you had experience with c-binding, this may not to be a bad choice :)

Here we give an example of how to create and sign transactions with option 1 in Java. Please follow the guide at [Transaction Builder Generator][12] to generate code into your project.

**Example**: create and sign a transaction that transfers 12 Coint1 coins from account1 to account2.

```Java

ChainId testNetChainID = new ChainId((byte) 2); // Testnet chain id is static value
String currencyCode = "XUS";
String account1_address = "a74fd7c46952c497e75afb0a7932586d";
String account1_public_key = "447fc3be296803c2303951c7816624c7566730a5cc6860a4a1bd3c04731569f5";
String account1_private_key = "cd9a2c90296a210249128ae3c908611637b2e00efd4986670e252abf3fabd1a9";
String account2_address = "5b9f7691937732eedfbe4f194275247b";
long amountToTransfer = coins(12);

// step 1: create transaction script:
TypeTag currencyCodeMoveResource = new TypeTag.Struct(new StructTag(
        bytesToAddress(hexToBytes("00000000000000000000000000000001")), // 0x1 is core code account address
        new Identifier(currencyCode),
        new Identifier(currencyCode),
        new ArrayList<>()
));

Script script = Helpers.encode_peer_to_peer_with_metadata_script( // Helpers.encode_xxx is code generated by transaction builder generator
        currencyCodeMoveResource,
        hexToAddress(account2_address),
        amountToTransfer,
        new Bytes(new byte[]{}),
        new Bytes(new byte[]{})
);

// step 2: get current submitting transaction account sequence number.
Account account1Data = client.getAccount(account1_address);

// step 3: create RawTransaction
RawTransaction rt = new RawTransaction(
        hexToAddress(account1_address),
        account1Data.sequence_number,
        new TransactionPayload.Script(script),
        coins(1),                 // maxGasAmount
        0L,                       // gasUnitPrice, you can always set gas unit price to zero on Testnet. At launch, gas unit price can be zero in most of time. Only during high congestion, you may specify a gas price.
        currencyCode,
        System.currentTimeMillis()/1000 + 30, // expirationTimestampSecs, expire after 30 seconds
        testNetChainID
);

byte[] rawTxnBytes = toBCS(rt);

```

You can find imports and util functions code [here](#util-functions).

The following code does signing transaction:

```Java

// sha3 hash "DIEM::RawTransaction" bytes first, then concat with raw transaction bytes to create a message for signing.
byte[] hash = concat(sha3Hash("DIEM::RawTransaction".getBytes()), rawTxnBytes);

// [bouncycastle](https://www.bouncycastle.org/)'s Ed25519Signer
Ed25519Signer signer = new Ed25519Signer();
byte[] privateKeyBytes = hexToBytes(account1_private_key);
signer.init(true, new Ed25519PrivateKeyParameters(privateKeyBytes, 0));
signer.update(hash, 0, hash.length);
byte[] sign = signer.generateSignature();

SignedTransaction st = new SignedTransaction(rt, new TransactionAuthenticator.Ed25519(
        new Ed25519PublicKey(new Bytes(hexToBytes(account1_public_key))),
        new Ed25519Signature(new Bytes(sign))
));
String signedTxnData = bytesToHex(toBCS(st));

```

For more details related to Diem crypto, please checkout [Crypto Spec](../../specifications/crypto/README.md).

When you implement above logic, you may extract `createRawTransaction` and `createSignedTransaction` methods and use the following data to confirm their logic is correct:

1. Given the account sequence number: 0.
2. Given expirationTimestampSecs to 1997844332.
3. Keep other data same, you should get:
   1. hex-encoded raw transaction BCS serialized bytes: A74FD7C46952C497E75AFB0A7932586D000000000000000001E101A11CEB0B010000000701000202020403061004160205181D0735610896011000000001010000020001000003020301010004010300010501060C0108000506080005030A020A020005060C05030A020A020109000C4C696272614163636F756E741257697468647261774361706162696C6974791B657874726163745F77697468647261775F6361706162696C697479087061795F66726F6D1B726573746F72655F77697468647261775F6361706162696C69747900000000000000000000000000000001010104010C0B0011000C050E050A010A020B030B0438000B05110202010700000000000000000000000000000001034C4252034C42520004035B9F7691937732EEDFBE4F194275247B01001BB700000000000400040040420F00000000000000000000000000034C42526CAF14770000000002
   2. hex-encoded signed transaction BCS serialized bytes: A74FD7C46952C497E75AFB0A7932586D000000000000000001E101A11CEB0B010000000701000202020403061004160205181D0735610896011000000001010000020001000003020301010004010300010501060C0108000506080005030A020A020005060C05030A020A020109000C4C696272614163636F756E741257697468647261774361706162696C6974791B657874726163745F77697468647261775F6361706162696C697479087061795F66726F6D1B726573746F72655F77697468647261775F6361706162696C69747900000000000000000000000000000001010104010C0B0011000C050E050A010A020B030B0438000B05110202010700000000000000000000000000000001034C4252034C42520004035B9F7691937732EEDFBE4F194275247B01001BB700000000000400040040420F00000000000000000000000000034C42526CAF147700000000020020447FC3BE296803C2303951C7816624C7566730A5CC6860A4A1BD3C04731569F5400A40B32AA9EB1C16F48E78F01AE3EE90AEDFABF259F0E74058639E46F082FF475FD24A93DE4BD9D701ACB8E31660BCB301065216D41A0DBAA8DB5CC63A528D05


#### Util Functions


```Java

import com.novi.bcs.BcsSerializer;
import com.novi.serde.Bytes;
import com.novi.serde.Serializer;
import org.bouncycastle.crypto.params.Ed25519PrivateKeyParameters;
import org.bouncycastle.crypto.signers.Ed25519Signer;
import org.diem.stdlib.Helpers;
import org.diem.types.*;

import java.io.IOException;
import java.util.ArrayList;

// ......

public static long coins(long n) {
    return n * 1000000;
}

public static AccountAddress hexToAddress(String hex) {
    return bytesToAddress(hexToBytes(hex));
}

static AccountAddress bytesToAddress(byte[] values) {
    assert values.length == 16;
    Byte[] address = new Byte[16];
    for (int i = 0; i < 16; i++) {
        address[i] = Byte.valueOf(values[i]);
    }
    return new AccountAddress(address);
}

public static byte[] hexToBytes(String hex) {
    return BaseEncoding.base16().decode(hex.toUpperCase());
}

public static String bytesToHex(byte[] bytes) {
    return BaseEncoding.base16().encode(bytes);
}

public static String bytesToHex(Bytes bytes) {
    return bytesToHex(bytes.content());
}

public static byte[] toBCS(RawTransaction rt) throws Exception {
    Serializer serializer = new BcsSerializer();
    rt.serialize(serializer);
    return serializer.get_bytes();
}

public static byte[] toBCS(SignedTransaction rt) throws Exception {
    Serializer serializer = new BcsSerializer();
    rt.serialize(serializer);
    return serializer.get_bytes();
}

public static byte[] sha3Hash(byte[] data) {
    SHA3.DigestSHA3 digestSHA3 = new SHA3.Digest256();
    return digestSHA3.digest(data);
}

public static byte[] concat(byte[] part1, byte[] part2) {
    byte[] ret = new byte[part1.length + part2.length];
    System.arraycopy(part1, 0, ret, 0, part1.length);
    System.arraycopy(part2, 0, ret, part1.length, part2.length);
    return ret;
}

public static String addressToHex(AccountAddress address) {
    byte[] bytes = new byte[16];
    for (int i = 0; i < 16; i++) {
        bytes[i] = Byte.valueOf(address.value[i]);
    }
    return bytesToHex(bytes);
}

```

#### Submit transaction

After extracting out creating and signing transaction logic, the JSON-RPC [submit](method_submit.md) method API itself is simple with hex-encoded signed transaction serialized string.
Assuming we have the API implemented by `client` like other `get` methods, we call `submit` with the `signedTxnData` to send the transaction to the server.

```Java
client.submit(signedTxnData);
```

#### Wait for transaction executed and validate result

After the transaction is submitted successfully, we need to wait for it to be executed and validate the execution result.

We can call [get_account_transaction](method_get_account_transaction.md) to find the transaction by account address and sequence number.
If transaction has not been executed yet, server responses null:

```Java
public Transaction waitForTransaction(String address, @Unsigned long sequence, String transactionHash,
    @Unsigned long expirationTimeSec, int timeout) throws DiemException {

    Calendar calendar = Calendar.getInstance();
    calendar.add(Calendar.SECOND, timeout);
    Date maxTime = calendar.getTime();

    while (Calendar.getInstance().getTime().before(maxTime)) {
        Transaction accountTransaction = getAccountTransaction(address, sequence, true);

        if (accountTransaction != null) {
            if (!accountTransaction.getHash().equalsIgnoreCase(transactionHash)) {
                throw new DiemException(
                        String.format("found transaction, but hash does not match, given %s, but got %s",
                                transactionHash, accountTransaction.getHash()));
            }
            if (!txn.getVmStatus() != null && "executed".equalsIgnoreCase(accountTransaction.getVmStatus().getType())) {
                throw new DiemTransactionExecutionFailedException(
                        String.format("transaction execution failed: %s", accountTransaction.getVmStatus()));
            }

            return accountTransaction;
        }
        try {
            Thread.sleep(200);
        } catch (InterruptedException e) {
            throw new RuntimeException(e);
        }
    }

    throw new DiemWaitForTransactionTimeoutException(
            String.format("transaction not found within timeout period: %d (seconds)", timeout));
}
```

In above example code, we do the following 2 validations after the transaction showed up:
1. transaction#hash should be the same with the hash we created for `SignedTransaction`. This makes sure the transaction we got is the one we submitted.
2. transaction#vm_status#type should be "executed". Type "executed" means the transaction is executed successfully, all other types are failures. See [VMStatus](type_transaction.md#type-vmstatus) doc for more details.

We also should have a wait timeout for the case if the transaction is dropped some where for unknown reason.

### Error Handling

There are four general types errors you need consider:
- Transport layer error, e.g. HTTP call failure.
- JSON-RPC protocol error: e.g. server response non json data, or can't be parsed into [Diem JSON-RPC SPEC](./../json-rpc-spec.md) defined data structure, or missing result & error field.
- JSON-RPC error: error returned from server.
- Invalid arguments error: the caller of your client API may provide invalid arguments like invalid hex-encoded account address.

Distinguish above four types errors can help application developer to decide what to do with each different type error:
- Application may consider retry for transport layer error.
- JSON-RPC protocol error indicates a server side bug.
- Invalid arguments error indicates application code bug.
- JSON-RPC error has 2 major groups errors:
   - Invalid request: it indicates client side error, either it's application code bug or the client (your code) bug. If you did well with handling invalid arguments, then it means your client code has bug.
   - Server error: this can be a server side bug, or important information related to submitted transaction validation or execution error.

Other than general error handling, another type of error that client / application should pay attention to is server side stale response. This type problem happens when a Full Node is out of sync with the Diem network, or you connected to a sync delayed Full Node in a cluster of Full Nodes. To prevent these problems, we need:
- Track server side data freshness, Diem JSON-RPC server will always respond `diem_ledger_version` and `diem_ledger_timestampusec` (see [Diem Extensions](./../json-rpc-spec.md#diem-extensions)) for clients to validate and track server side data freshness.
- Retry query / get methods calls when response is from a stale server.
- Do not retry for submit transaction call, because the submitted transaction can be synced correctly even you submitted it to a stale server. You may receive a JSON-RPC error if submitted same transaction.

### More

Once the above basic function works, you have a minimum client ready for usage.
To make a production quality client, please checkout our [Client CHECKLIST](client_checklist.md).

[1]: https://developers.diem.com/docs/rustdocs/diem_types/transaction/struct.SignedTransaction.html "SignedTransaction"
[2]: ../../language/transaction-builder/generator/README.md "Transaction Builder Generator"
[3]: ./../../client/swiss-knife/README.md "Diem Swiss Knife"
[4]: https://developers.diem.com/docs/rustdocs/diem_types/transaction/struct.RawTransaction.html "RawTransaction"
[5]: https://docs.rs/bcs/ "BCS"
[6]: ./../../client/swiss-knife#generate-a-ed25519-keypair "Swiss Knife Gen Keys"
[7]: ./../../language/stdlib/transaction_scripts/doc/peer_to_peer_with_metadata.md#function-peer_to_peer_with_metadata-1 "P2P script doc"
[8]: ./../../client/swiss-knife/README.md#examples-for-generate-raw-txn-and-generate-signed-txn-operations "Swiss Knife gen txn"
[9]: ./../../client/swiss-knife/README.md#building-the-binary-in-a-release-optimized-mode "Swiss Knife binary"
[10]: ../../language/transaction-builder/generator/README.md#supported-languages "Transaction Builder Generator supports"
[11]: ./../../client/diem-dev/include/data.h "C binding head file"
[12]: ../../language/transaction-builder/generator/README.md#java "Generate Java Txn Builder"
