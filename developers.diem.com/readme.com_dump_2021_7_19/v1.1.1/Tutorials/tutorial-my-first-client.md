---
title: "My first client"
slug: "tutorial-my-first-client"
hidden: false
metadata: 
  title: "My first client"
  description: "Build your first client for the Diem Blockchain using our official SDKs."
createdAt: "2021-02-11T01:43:14.606Z"
updatedAt: "2021-04-21T21:43:32.823Z"
---
This tutorial is an introduction to client development on testnet using the official SDKs.


## Getting Started

In this tutorial, we demonstrate the key elements of a basic client using the official SDKs to interact with the Blockchain. The code for the tutorial is available here: [my-first-client](https://github.com/diem/my-first-client). The code in this project can be run from the root of the project directory by issuing the `make` command.

The example code uses the official Client SDKs. Currently, Go, Java, and Python are available. These libraries are developed to simplify aspects of the development process. If your language is not currently supported, or on the upcoming roadmap (Rust), then you will want to  refer to the low-level JSON-RPC API. To request additional functionality or to track when it is implemented, you can submit a GitHub issue on the corresponding project repository.

To see advanced usage, refer to the Reference Wallet project.


## Setup

All code examples are shared in the [my-first-client](https://github.com/diem/my-first-client) repo on GitHub.

### Clone the repo:


[block:code]
{
  "codes": [
    {
      "code": "git clone [https://github.com/diem/my-first-client.git](https://github.com/diem/my-first-client.git)",
      "language": "text",
      "name": ""
    }
  ]
}
[/block]
Each SDK has the following system requirements:

*   Java: Java 8+
*   Python: Python v3.7+, pipenv

### Run the examples:

`make`

or

`make python` for Python

`make java` for Java

## Connecting to the network

The first thing your client code will need to do is connect to the network.
[block:code]
{
  "codes": [
    {
      "code": "client = testnet.create_client();",
      "language": "python"
    },
    {
      "code": "client = Testnet.createClient();",
      "language": "java"
    }
  ]
}
[/block]
## Creating wallets

Wallets are addresses on the Blockchain that may issue transactions that send or receive funds. Private and Public key are needed.
[block:code]
{
  "codes": [
    {
      "code": "# generate private key for sender account\nsender_private_key = Ed25519PrivateKey.generate()\n\n# generate auth key for sender account\nsender_auth_key = AuthKey.from_public_key(sender_private_key.public_key())\nprint(f\"Generated sender address: {utils.account_address_hex(sender_auth_key.account_address())}\")",
      "language": "python"
    },
    {
      "code": "PrivateKey senderPrivateKey = new Ed25519PrivateKey(new Ed25519PrivateKeyParameters(new SecureRandom()));\nAuthKey senderAuthKey = AuthKey.ed24419(senderPrivateKey.publicKey());",
      "language": "java"
    }
  ]
}
[/block]
## Requesting XUS from the faucet

A faucet is a blockchain tool for testing blockchain transactions by providing coins for testing. When instructed, the testnet faucet will send the requested amount of XUS to the address provided.

Here, we request 100 XUS from the faucet and deposit it in Wallet A.
[block:code]
{
  "codes": [
    {
      "code": "faucet = testnet.Faucet(client)\nfaucet.mint(sender_auth_key.hex(), 10000000, \"XUS\")",
      "language": "python"
    },
    {
      "code": "Testnet.mintCoins(client, 10000000, senderAuthKey.hex(), \"XUS\");",
      "language": "java"
    }
  ]
}
[/block]
## Getting a balance

In the previous step we requested 100 XUS from the faucet. We can verify that our test wallet now has the expected balance.
[block:code]
{
  "codes": [
    {
      "code": "# connect to testnet\nclient = testnet.create_client()\n\n# generate private key\nprivate_key = Ed25519PrivateKey.generate()\n\n# generate auth key\nauth_key = AuthKey.from_public_key(private_key.public_key())\nprint(f\"Generated address: {utils.account_address_hex(auth_key.account_address())}\")\n\n# create account\nfaucet = testnet.Faucet(client)\nfaucet.mint(auth_key.hex(), 1340000000, \"XUS\")\n\n# get account information\naccount = client.get_account(auth_key.account_address())\nprint(\"Account info:\")\nprint(account)",
      "language": "python"
    },
    {
      "code": "//connect to testnet\nDiemClient client = Testnet.createClient();\n\n//generate private key for new account\nPrivateKey privateKey = new Ed25519PrivateKey(new Ed25519PrivateKeyParameters(new SecureRandom()));\n\n//generate auth key for new account\nAuthKey authKey = AuthKey.ed24419(privateKey.publicKey());\n\n//create account\nTestnet.mintCoins(client, 10000000, authKey.hex(), CURRENCY_CODE);\n\n//get account information\nAccount account = client.getAccount(authKey.accountAddress());\nSystem.out.println(\"Account info:\");\nSystem.out.println(account);",
      "language": "java"
    }
  ]
}
[/block]
## Sending Coins

_Note: There are several types of peer to peer transactions. These transactions shall have regulatory and compliance requirements that must be followed. To learn more about requirements, please visit the Prospective VASPs document here._

Next we demonstrate sending 10 units of XUS from a Sender wallet to a Receiver wallet.
[block:code]
{
  "codes": [
    {
      "code": "# connect to testnet\nclient = testnet.create_client()\n\n# generate private key for sender account\nsender_private_key = Ed25519PrivateKey.generate()\n\n# generate auth key for sender account\nsender_auth_key = AuthKey.from_public_key(sender_private_key.public_key())\nprint(f\"Generated sender address: {utils.account_address_hex(sender_auth_key.account_address())}\")\n\n# create sender account\nfaucet = testnet.Faucet(client)\ntestnet.Faucet.mint(faucet, sender_auth_key.hex(), 100000000, \"XUS\")\n\n# get sender account\nsender_account = client.get_account(sender_auth_key.account_address())\n\n# generate private key for receiver account\nreceiver_private_key = Ed25519PrivateKey.generate()\n\n# generate auth key for receiver account\nreceiver_auth_key = AuthKey.from_public_key(receiver_private_key.public_key())\nprint(f\"Generated receiver address: {utils.account_address_hex(receiver_auth_key.account_address())}\")\n\n# create receiver account\nfaucet = testnet.Faucet(client)\nfaucet.mint(receiver_auth_key.hex(), 10000000, CURRENCY)\n\n# create script\nscript = stdlib.encode_peer_to_peer_with_metadata_script(\n    currency=utils.currency_code(CURRENCY),\n    payee=receiver_auth_key.account_address(),\n    amount=10000000,\n    metadata=b'',  # no requirement for metadata and metadata signature\n    metadata_signature=b'',\n    )\n\n# create transaction\nraw_transaction = diem_types.RawTransaction(\n    sender=sender_auth_key.account_address(),\n    sequence_number=sender_account.sequence_number,\n    payload=diem_types.TransactionPayload__Script(script),\n    max_gas_amount=1_000_000,\n    gas_unit_price=0,\n    gas_currency_code=CURRENCY,\n    expiration_timestamp_secs=int(time.time()) + 30,\n    chain_id=CHAIN_ID,\n    )\n\n# sign transaction\nsignature = sender_private_key.sign(utils.raw_transaction_signing_msg(raw_transaction))\npublic_key_bytes = utils.public_key_bytes(sender_private_key.public_key())\nsigned_txn = utils.create_signed_transaction(raw_transaction, public_key_bytes, signature)\n\n# submit transaction\nclient.submit(signed_txn)\n\n# wait for transaction\nclient.wait_for_transaction(signed_txn)",
      "language": "python"
    },
    {
      "code": "//connect to testnet\nDiemClient client = Testnet.createClient();\n\n//generate private key for sender account\nPrivateKey senderPrivateKey = new Ed25519PrivateKey(new Ed25519PrivateKeyParameters(new SecureRandom()));\n\n//generate auth key for sender account\nAuthKey senderAuthKey = AuthKey.ed24419(senderPrivateKey.publicKey());\n\n//create sender account with 100 XUS balance\nTestnet.mintCoins(client, 100000000, senderAuthKey.hex(), CURRENCY_CODE);\n\n//get sender account for sequence number\nAccount account = client.getAccount(senderAuthKey.accountAddress());\n\n//generate private key for receiver account\nPrivateKey receiverPrivateKey = new Ed25519PrivateKey(new Ed25519PrivateKeyParameters(new SecureRandom()));\n\n//generate auth key for receiver account\nAuthKey receiverAuthKey = AuthKey.ed24419(receiverPrivateKey.publicKey());\n\n//create receiver account with 1 XUS balance\nTestnet.mintCoins(client, 10000000, receiverAuthKey.hex(), CURRENCY_CODE);\n\n//create script\nTransactionPayload script = new TransactionPayload.Script(\n        Helpers.encode_peer_to_peer_with_metadata_script(\n                CurrencyCode.typeTag(CURRENCY_CODE),\n                receiverAuthKey.accountAddress(),\n                10000000L,\n                new Bytes(new byte[0]),\n                new Bytes(new byte[0])));\n\n//create transaction to send 1 XUS\nRawTransaction rawTransaction = new RawTransaction(\n        senderAuthKey.accountAddress(),\n        account.getSequenceNumber(),\n        script,\n        1000000L,\n        0L,\n        CURRENCY_CODE,\n        (System.currentTimeMillis() / 1000) + 300,\n        CHAIN_ID);\n\n//sign transaction\nSignedTransaction st = Signer.sign(senderPrivateKey, rawTransaction);\n\n//submit transaction\ntry {\n    client.submit(st);\n} catch (StaleResponseException e) {\n    //ignore\n}\n\n//wait for the transaction to complete\nTransaction transaction = client.waitForTransaction(st, 100000);\nSystem.out.println(transaction);",
      "language": "java"
    }
  ]
}
[/block]
We can verify that 10 XUS was sent by verifying the sender’s wallet balance is 90 and receiver’s balance is 10.


## Transaction Intent (DIP-5)

_DIP-5 introduces a standard way for communicating information about a specific transaction. DIP-5 encodes a destination address, currency, and amount. You can use this information to handle the user’s intent._

Each SDK provides the following helper function for processing this information.
[block:code]
{
  "codes": [
    {
      "code": "//generate private key for new account\nPrivateKey privateKey = new Ed25519PrivateKey(new Ed25519PrivateKeyParameters(new SecureRandom()));\n\n//generate auth key for new account\nAuthKey authKey = AuthKey.ed24419(privateKey.publicKey());\n\n//create IntentIdentifier\nAccountIdentifier accountIdentifier = new AccountIdentifier(TestnetPrefix, authKey.accountAddress());\nIntentIdentifier intentIdentifier = new IntentIdentifier(accountIdentifier, CURRENCY, 10000000L);\nString intentIdentifierString = intentIdentifier.encode();\nSystem.out.println(\"Encoded IntentIdentifier: \" + intentIdentifierString);\n\n//deserialize IntentIdentifier\nIntentIdentifier decodedIntentIdentifier = decode(TestnetPrefix, intentIdentifierString);\n\nSystem.out.println(\"Account (HEX) from intent: \" + Hex.encode(decodedIntentIdentifier.getAccountIdentifier().getAccountAddress().value));\nSystem.out.println(\"Amount from intent: \" + decodedIntentIdentifier.getAmount());\nSystem.out.println(\"Currency from intent: \" + decodedIntentIdentifier.getCurrency());",
      "language": "python"
    },
    {
      "code": "# generate private key\nprivate_key = Ed25519PrivateKey.generate()\n\n# generate auth key\nauth_key = AuthKey.from_public_key(private_key.public_key())\n\n# create intent identifier\naccount_identifier = identifier.encode_account(utils.account_address_hex(auth_key.account_address()), None, identifier.TLB)\nencoded_intent_identifier = identifier.encode_intent(account_identifier, \"XUS\", 10000000)\nprint(f\"Encoded IntentIdentifier: {encoded_intent_identifier}\")\n\n# deserialize IntentIdentifier\nintent_identifier = diem.identifier.decode_intent(encoded_intent_identifier, identifier.TLB)\nprint(f\"Account (HEX) from intent: {utils.account_address_hex(intent_identifier.account_address)}\")\nprint(f\"Amount from intent: {intent_identifier.amount}\")\nprint(f\"Currency from intent: {intent_identifier.currency_code}\")",
      "language": "java"
    }
  ]
}
[/block]
## Subscribing to events

To monitor and react to transactions, you may subscribe to events.

In the example below, we will setup a wallet with 100 XUS and then call the mint to add 1 XUS ten times.
[block:code]
{
  "codes": [
    {
      "code": "import time\nfrom random import randrange\nfrom threading import Thread\nfrom cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey\nfrom diem import testnet, AuthKey, utils\n\nCURRENCY = \"XUS\"\n\n\"\"\"get_events_example demonstrates how to subscribe to a specific events stream base on events key\"\"\"\n\ndef main():\n\n    # connect to testnet\n    client = testnet.create_client()\n\n    # generate private key\n    private_key = Ed25519PrivateKey.generate()\n\n    # generate auth key\n    auth_key = AuthKey.from_public_key(private_key.public_key())\n    print(f\"Generated address: {utils.account_address_hex(auth_key.account_address())}\")\n\n    # create new account\n    faucet = testnet.Faucet(client)\n    faucet.mint(auth_key.hex(), 100000000, CURRENCY)\n\n    # get account events key\n    account = client.get_account(auth_key.account_address())\n    events_key = account.received_events_key\n\n    # start minter to demonstrates events creation\n    start_minter(client, auth_key)\n\n    # demonstrates events subscription\n    subscribe(client, events_key)\n\ndef subscribe_(client, events_key):\n    start = 0\n    for x in range(0, 15):\n        events = client.get_events(events_key, start, 10)\n        start += len(events)\n        print(f\"{len(events)} new events found\")\n        time.sleep(3)\n        for i in range(0, len(events)):\n            print(f\"Event # {i + 1}:\")\n            print(events[i])\n\ndef minter(client, auth_key):\n    for x in range(0, 10):\n        amount = 1000000\n        faucet = testnet.Faucet(client)\n        testnet.Faucet.mint(faucet, auth_key.hex(), amount, CURRENCY)\n        time.sleep(1)\n\ndef subscribe(client, events_key):\n    Thread(target=subscribe_, args=(client, events_key,)).start()\n\ndef start_minter(client, auth_key):\n    Thread(target=minter, args=(client, auth_key)).start()\n\nif __name__ == \"__main__\":\n    main()",
      "language": "python",
      "name": "Python"
    },
    {
      "code": "client/blob/master/java/src/main/java/example/GetEventsExample.java\"\n\npackage example;\nimport org.bouncycastle.crypto.params.Ed25519PrivateKeyParameters;\nimport org.diem.*;\nimport org.diem.jsonrpctypes.JsonRpc.Account;\nimport org.diem.jsonrpctypes.JsonRpc.Event;\nimport java.security.SecureRandom;\nimport java.util.List;\nimport java.util.concurrent.ThreadLocalRandom;\n\n/**\n * GetEventsExample demonstrates how to subscribe to a specific events stream base on events key\n */\n\npublic class GetEventsExample {\n   public static final String CURRENCY_CODE = \"XUS\";\n   public static void main(String[] args) throws DiemException {\n       //connect to testnet\n       DiemClient client = Testnet.createClient();\n       //create new account\n       SecureRandom random = new SecureRandom();\n       Ed25519PrivateKeyParameters privateKeyParams = new Ed25519PrivateKeyParameters(random);\n       Ed25519PrivateKey privateKey = new Ed25519PrivateKey(privateKeyParams);\n       AuthKey authKey = AuthKey.ed24419(privateKey.publicKey());\n       Testnet.mintCoins(client, 100000000, authKey.hex(), CURRENCY_CODE);\n       //get account events key\n       Account account = client.getAccount(authKey.accountAddress());\n       String eventsKey = account.getReceivedEventsKey();\n       //start minter to demonstrates events creation\n       startMinter(client, authKey);\n       //demonstrates events subscription\n       subscribe(client, eventsKey);\n   }\n   public static void subscribe(DiemClient client, String eventsKey) {\n       Runnable listener = () -> {\n           long start = 0;\n           for (int i = 0; i &lt; 15; i++) {\n               List&lt;Event> events;\n               try {\n                   events = client.getEvents(eventsKey, start, 10);\n               } catch (DiemException e) {\n                   throw new RuntimeException(e);\n               }\n               start += events.size();\n               System.out.println(events.size() + \" new events found\");\n               for (int j = 0; j &lt; events.size(); j++) {\n                   System.out.println(\"Event #\" + (j + 1) + \":\");\n                   System.out.println(events.get(j));\n               }\n               try {\n                   Thread.sleep(3_000);\n               } catch (InterruptedException e) {\n                   throw new RuntimeException(e);\n               }\n           }\n       };\n       Thread listenerThread = new Thread(listener);\n       listenerThread.start();\n   }\n   private static void startMinter(DiemClient client, AuthKey authKey) {\n       Runnable minter = () -> {\n           for (int i = 0; i &lt; 10; i++) {\n               int amount =  1000000;\n               Testnet.mintCoins(client, amount, authKey.hex(), CURRENCY_CODE);\n               try {\n                   Thread.sleep(1_000);\n               } catch (InterruptedException e) {\n                   throw new RuntimeException(e);\n               }\n           }\n       };\n       Thread minterThread = new Thread(minter);\n       minterThread.start();\n   }\n}",
      "language": "java",
      "name": "Java"
    }
  ]
}
[/block]