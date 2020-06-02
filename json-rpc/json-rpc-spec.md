# CONTENT

**Note**: The Libra Client API is currently under development and may be updated in the future.


## Overview

The Libra client API is based on the JSON-RPC protocol. This specification defines the client API endpoints and types, and provides usage examples.


## JSON-RPC specification

JSON-RPC is a stateless, light-weight remote procedure call (RPC) protocol. Refer to the [JSON-RPC Specification](https://www.jsonrpc.org/specification) for further details.


### Batched requests

The JSON-RPC protocol allows requests to be batched. An arbitrary number of requests can be combined into a single batch and submitted to the server. These requests will be processed together under a single request context.


### Errors

If errors occur during a request, they are returned in an error object, as defined in: [https://www.jsonrpc.org/specification#error_object](https://www.jsonrpc.org/specification#error_object)

Unless specifically mentioned below, Libra JSON-RPC will return the default error code - 32000 for generic server-side errors. More information may be returned in the ‘message’ and the ‘data’ fields, but this is not guaranteed.



---



## **submit** - method

**Description**

Submit a signed transaction to a full node.


### Parameters


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>data</strong>
   </td>
   <td>string
   </td>
   <td>Signed transaction data - hex-encoded bytes of serialized Libra SignedTransaction type.
   </td>
  </tr>
</table>



### Returns

Null - on success


### Errors

Errors during the transaction are indicated by different error codes:


<table>
  <tr><td>-32000</td><td>Default server error</td></tr>
  <tr><td>-32001</td><td>VM validation error</td></tr>
  <tr><td>-32002</td><td>VM verification error</td></tr>
  <tr><td>-32003</td><td>VM invariant violation error</td></tr>
  <tr><td>-32004</td><td>VM deserialization error</td></tr>
  <tr><td>-32005</td><td>VM execution error</td></tr>
  <tr><td>-32006</td><td>VM unknown error</td></tr>
  <tr><td>-32007</td><td>Mempool error: invalid sequence number</td></tr>
  <tr><td>-32008</td><td>Mempool is full error</td></tr>
  <tr><td>-32009</td><td>Mempool error: account reached max capacity per account</td></tr>
  <tr><td>-32010</td><td>Mempool error: invalid update (only gas price increase is allowed)</td></tr>
  <tr><td>-32011</td><td>Mempool error: transaction did not pass VM validation</td></tr>
  <tr><td>-32012</td><td>Unknown error</td></tr>
</table>

More information might be available in the “message” field, but this is not guaranteed.


### Example


```
// Request: submits a transaction whose hex-encoded LCS byte representation is in params
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"submit","params":["c1fda0ec67c1b87bfb9e883e2080e530040000000000000002f401a11ceb0b010007014600000002000000034800000011000000045900000004000000055d0000001c00000007790000004900000008c20000001000000009d200000022000000000000010001010100020203000003040101010006020602050a0200010501010405030a020a0205050a02030a020a020109000c4c696272614163636f756e74166372656174655f746573746e65745f6163636f756e74066578697374731d7061795f66726f6d5f73656e6465725f776974685f6d6574616461746100000000000000000000000000000000010105010e000a001101200305000508000a000b0138000a000a020b030b04380102010700000000000000000000000000000000034c425201540005034ac94d88e90acd4cf0294e898e421e9404106484f428e88bba93de5053e051acb6ec0180969800000000000400040040420f00000000000000000000000000034c42528c8dd15e000000000020500a9002995e1af93bbdaf977385ed507b174bb3dc6936efd72612d56198a19d4084766159c40c4e91e61e3dbc30a9553c3111a7a582981d57ce4dff0aa383a529209b15046850c5115ef16b0b2e0c861be71186effb1b5707495e12ca06fe660a"],"id": 1}'

// Response, for successful transaction submission
{
  "id":1,
  "jsonrpc": "2.0",
  "result": null
}
```




---



## **get_transactions** - method

**Description**

Get the transactions on the blockchain


### Parameters


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>start_version</strong>
   </td>
   <td>u64
   </td>
   <td>Start on this transaction version for this query
   </td>
  </tr>
  <tr>
   <td><strong>limit</strong>
   </td>
   <td>u64
   </td>
   <td>Limit the number of transactions returned
   </td>
  </tr>
  <tr>
   <td><strong>include_events</strong>
   </td>
   <td>bool
   </td>
   <td>Set to true, to also fetch events for each transaction
   </td>
  </tr>
</table>



### Returns

Array of [Transaction](#transaction---type) objects

if include_events is false, the events field in the Transaction object will be an empty array.


### Example


```
// Request: fetches 10 transactions since version 1000
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_transactions","params":[4433485, 10, false],"id":1}'

// Response
{
    "id":1,
    "jsonrpc":"2.0",
    "result":[
        {
            "events":[

            ],
            "gas_used":0,
            "transaction":{
                "expiration_time":1590680747,
                "gas_unit_price":0,
                "max_gas_amount":1000000,
                "public_key":"500a9002995e1af93bbdaf977385ed507b174bb3dc6936efd72612d56198a19d",
                "script":{
                    "amount":10000000,
                    "auth_key_prefix":"6484f428e88bba93de5053e051acb6ec",
                    "metadata":"",
                    "metadata_signature":"",
                    "receiver":"4ac94d88e90acd4cf0294e898e421e94",
                    "type":"peer_to_peer_transaction"
                },
                "script_hash":"c8bc3dda60e9662965b3223c22e3d3e3e7b6f698cf1a6930a449eb99daa35e7c",
                "sender":"c1fda0ec67c1b87bfb9e883e2080e530",
                "sequence_number":0,
                "signature":"fe335285e5d87db25f86041d033414bfdf77ddae6f0dfbdc65ff4f5965ff810ef9c85ce00ede0820ce0cf5903f9ab3e93fa6e49bbf770aba9b083a985361fa01",
                "signature_scheme":"Scheme::Ed25519",
                "type":"user"
            },
            "version":4433485,
            "vm_status":4001
        },
        ....
    ]
}
```



##

---



## **get_account_state** - method

**Description**

Get the latest account state for a given account.


### Parameters


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>account</strong>
   </td>
   <td>string
   </td>
   <td>Hex-encoded account address.
   </td>
  </tr>
</table>



### Returns

[Account](#account---type) - If account exists

Null - If account does not exist


### Example

```
// Request: fetches account state for account address "0xc1fda0ec67c1b87bfb9e883e2080e530"
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_account_state","params":["c1fda0ec67c1b87bfb9e883e2080e530"],"id":1}'


// Response
{
   "id":1,
   "jsonrpc":"2.0",
   "result":{
      "authentication_key":"bbdcca6b06aa596ca700cb23ba74cc8ec1fda0ec67c1b87bfb9e883e2080e530",
      "balances":[
         {
            "amount":200000000,
            "currency":"LBR"
         }
      ],
      "delegated_key_rotation_capability":false,
      "delegated_withdrawal_capability":false,
      "received_events_key":"0000000000000000c1fda0ec67c1b87bfb9e883e2080e530",
      "sent_events_key":"0100000000000000c1fda0ec67c1b87bfb9e883e2080e530",
      "sequence_number":1
   }
}

// Sample Response for non-existent account
{
  "id":1,
  "jsonrpc": "2.0",
  "result": null,
}
```


##

---



## **get_account_transaction** - method

**Description**

Get the transaction sent by the account with the given sequence number


### Parameters


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>account</strong>
   </td>
   <td>string
   </td>
   <td>The account address, a hex-encoded string
   </td>
  </tr>
  <tr>
   <td><strong>sequence</strong>
   </td>
   <td>u64
   </td>
   <td>The account sequence number
   </td>
  </tr>
  <tr>
   <td><strong>include_events</strong>
   </td>
   <td>bool
   </td>
   <td>Set to true to also fetch events generated by the transaction
   </td>
  </tr>
</table>



### Returns

[Transaction](#transaction---type) - If transaction exists

Null - If transaction does not exist


### Example


```
// Request: fetches transaction for account address "0xc1fda0ec67c1b87bfb9e883e2080e530" and sequence number 0, without including events associated with this transaction
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_account_transaction","params":["c1fda0ec67c1b87bfb9e883e2080e530", 0, false],"id":1}'

// Response
{
    "id":1,
    "jsonrpc":"2.0",
    "result":{
        "events":[

        ],
        "gas_used":0,
        "transaction":{
            "expiration_time":1590680747,
            "gas_unit_price":0,
            "max_gas_amount":1000000,
            "public_key":"500a9002995e1af93bbdaf977385ed507b174bb3dc6936efd72612d56198a19d",
            "script":{
                "amount":10000000,
                "auth_key_prefix":"6484f428e88bba93de5053e051acb6ec",
                "metadata":"",
                "metadata_signature":"",
                "receiver":"4ac94d88e90acd4cf0294e898e421e94",
                "type":"peer_to_peer_transaction"
            },
            "script_hash":"c8bc3dda60e9662965b3223c22e3d3e3e7b6f698cf1a6930a449eb99daa35e7c",
            "sender":"c1fda0ec67c1b87bfb9e883e2080e530",
            "sequence_number":0,
            "signature":"fe335285e5d87db25f86041d033414bfdf77ddae6f0dfbdc65ff4f5965ff810ef9c85ce00ede0820ce0cf5903f9ab3e93fa6e49bbf770aba9b083a985361fa01",
            "signature_scheme":"Scheme::Ed25519",
            "type":"user"
        },
        "version":4433485,
        "vm_status":4001
    }
}
```




---



## **get_metadata** - method

**Description**

Get the current blockchain metadata (e.g., current state of a Libra full node). All Read operations can be batched with get_metadata rpc to obtain synced metadata information with the request.

You can use this endpoint to verify liveness / status of nodes in the network:

get_metadata returns the latest transaction version and the block timestamp. If the timestamp or version is old (from the past), it means that the full node is not not up-to-date.


### Parameters

None


### Returns


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>version</strong>
   </td>
   <td>u64
   </td>
   <td>The latest transaction version
   </td>
  </tr>
  <tr>
   <td><strong>timestamp</strong>
   </td>
   <td>u64
   </td>
   <td>The block timestamp
   </td>
  </tr>
</table>



### Example


```
// Request: fetches current block metadata
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_metadata","params":[],"id":1}'

// Response
{
  "id":1,
  "jsonrpc": "2.0",
  "result": {
      "version": 100,
      "timestamp": 1584055164079210,
    }
}
```




---



## **get_events** - method

**Description**

Fetch the events for a given event stream.


### Parameters


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>key</strong>
   </td>
   <td>string
   </td>
   <td>Globally unique identifier of an event stream.
<p>
Note: For sent and received events, a client can use <a href="#get_account_state---method">get_account_state</a> to get the event key of the event streams for a given user.
   </td>
  </tr>
  <tr>
   <td><strong>start</strong>
   </td>
   <td>integer
   </td>
   <td>For this query, start at the event with this sequence number
   </td>
  </tr>
  <tr>
   <td><strong>limit</strong>
   </td>
   <td>integer
   </td>
   <td>Maximum number of events retrieved
   </td>
  </tr>
</table>



### Returns

Returns array of [Event](#event---type) objects


### Example


```
//Request: get events associated with event stream key "0100000000000000c1fda0ec67c1b87bfb9e883e2080e530"
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_events","params": ["0100000000000000c1fda0ec67c1b87bfb9e883e2080e530", 0, 10], "id":1}'

//Response
{
    "id":1,
    "jsonrpc":"2.0",
    "result":[
        {
            "data":{
                "amount":{
                    "amount":10000000,
                    "currency":"LBR"
                },
                "metadata":"",
                "receiver":"4ac94d88e90acd4cf0294e898e421e94",
                "type":"sentpayment"
            },
            "key":"0100000000000000c1fda0ec67c1b87bfb9e883e2080e530",
            "sequence_number":0,
            "transaction_version":4433485
        },
        {
            "data":{
                "amount":{
                    "amount":100000000,
                    "currency":"LBR"
                },
                "metadata":"",
                "receiver":"4ac94d88e90acd4cf0294e898e421e94",
                "type":"sentpayment"
            },
            "key":"0100000000000000c1fda0ec67c1b87bfb9e883e2080e530",
            "sequence_number":1,
            "transaction_version":10099706
        }
    ]
}
```




---



## **get_currencies** - method

**Description**

Get information about various currencies supported by the Libra blockchain


### Parameters

None


### Returns


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>code</strong>
   </td>
   <td>string
   </td>
   <td>Currency Code
   </td>
  </tr>
  <tr>
   <td><strong>fractional_part</strong>
   </td>
   <td>u64
   </td>
   <td>Max fractional part of single unit of currency allowed in a transaction
   </td>
  </tr>
  <tr>
   <td><strong>scaling_factor</strong>
   </td>
   <td>u64
   </td>
   <td>Factor by which the amount is scaled before it is stored in the blockchain
   </td>
  </tr>
</table>



### Example


```
// Request: fetches currencies supported by the system
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_currencies","params":[],"id":1}'

// Response
{
    "id": 1,
    "jsonrpc": "2.0",
    "result": [
        {
            "code": "LBR",
            "fractional_part": 1000,
            "scaling_factor": 1000000
        }
        ...
    ]
}
```


##

---



## Account - type

**Description**

A Libra account.


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>sequence_number
   </td>
   <td>u64
   </td>
   <td>The next sequence number for the current account
   </td>
  </tr>
  <tr>
   <td>authentication_key
   </td>
   <td>string
   </td>
   <td>Hex-encoded authentication key for the account
   </td>
  </tr>
  <tr>
   <td>delegated_key_rotation_capability
   </td>
   <td>bool
   </td>
   <td>If true, another account has the ability to rotate the authentication key for this account.
   </td>
  </tr>
  <tr>
   <td>delegated_withdrawal_capability
   </td>
   <td>bool
   </td>
   <td>If true, another account has the ability to withdraw funds from this account.
   </td>
  </tr>
  <tr>
   <td>balances
   </td>
   <td><a href="#amount---type">Amount</a>
   </td>
   <td>Balances of all the currencies associated with an account
   </td>
  </tr>
  <tr>
   <td>sent_events_key
   </td>
   <td>string
   </td>
   <td>Unique key for the sent events stream of this account
   </td>
  </tr>
  <tr>
   <td>received_events_key
   </td>
   <td>string
   </td>
   <td>Unique key for the received events stream of this account
   </td>
  </tr>
</table>


##

---



## Amount - type

### Attributes

<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>amount
   </td>
   <td>u64
   </td>
   <td>
    amount in currency microunits
   </td>
  </tr>
  <tr>
   <td>currency
   </td>
   <td>string
   </td>
   <td>currency string identifier
   </td>
  </tr>
</table>

##

---



## Transaction - type

**Description**

A transaction on the blockchain.


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>version
   </td>
   <td>u64
   </td>
   <td>The on-chain version or unique identifier of this transaction
   </td>
  </tr>
  <tr>
   <td>events
   </td>
   <td><a href="#event---type">Event</a>
   </td>
   <td>List of associated events. Empty for no events
   </td>
  </tr>
  <tr>
   <td>transaction_data
   </td>
   <td>Object
   </td>
   <td>Metadata for this transaction. Possible types are <a href="#BlockMetadataTransaction---type">BlockMetadataTransaction</a>, <a href="#WriteSetTransaction---type">WriteSetTransaction</a>, <a href="#UserTransaction---type">UserTransaction</a>, <a href="#UnknownTransaction---type">UnknownTransaction</a>. You should use the "type" field  to distinguish the type of the Object. (e.g., if "type" field is "user", this is a <a href="#UserTransaction---type">UserTransaction</a> object)
   </td>
  </tr>
  <tr>
   <td>vm_status
   </td>
   <td>u64
   </td>
   <td>S<a href="https://github.com/libra/libra/blob/master/types/src/vm_error.rs#L256">tatus code</a> representing the result of the VM processing this transaction.
   </td>
  </tr>
  <tr>
   <td>gas_used
   </td>
   <td>u64
   </td>
   <td>Amount of gas used by this transaction
   </td>
  </tr>
</table>



```

    {
      "version": 100,
      "transaction_data": {
         "type": "user"
         "sender": "0xc94770007dda54cF92009BFF0dE90c06F603a09f",
         "sequence_number": 0,
         "max_gas_amount": 7000,
         "gas_unit_price": 3,
         "expiration_time": 1582007787665718,
      },
      "events": [] // empty because include_events is set to false
    }
```



##

---



## BlockMetadataTransaction - type

**Description**

A Libra network transaction that contains the metadata for the block. This transaction is always at the beginning of a block.


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>type
   </td>
   <td>string
   </td>
   <td>Const string “blockmetadata”
   </td>
  </tr>
  <tr>
   <td>timestamp_usecs
   </td>
   <td>u64
   </td>
   <td>Timestamp for the current block, in microseconds
   </td>
  </tr>
</table>



##

---



## WriteSetTransaction - type

**Description**

A Libra network transaction that modifies storage data directly. Currently, no details are exposed in the API.


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>type
   </td>
   <td>string
   </td>
   <td>Const string “writeset”
   </td>
  </tr>
</table>




---



## UserTransaction - type

**Description**

User submitted transaction.


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>type
   </td>
   <td>string
   </td>
   <td>Const string “user”
   </td>
  </tr>
  <tr>
   <td>sender
   </td>
   <td>string
   </td>
   <td>Hex-encoded account address of the sender
   </td>
  </tr>
  <tr>
   <td>signature_scheme
   </td>
   <td>string
   </td>
   <td>Signature scheme used to sign this transaction
   </td>
  </tr>
  <tr>
   <td>signature
   </td>
   <td>string
   </td>
   <td>Hex-encoded signature of this transaction
   </td>
  </tr>
  <tr>
   <td>public_key
   </td>
   <td>string
   </td>
   <td>Hex-encoded public key of the transaction sender
   </td>
  </tr>
  <tr>
   <td>sequence_number
   </td>
   <td>u64
   </td>
   <td>Sequence number of this transaction corresponding to sender's account
   </td>
  </tr>
  <tr>
   <td>max_gas_amount
   </td>
   <td>u64
   </td>
   <td>Maximum amount of gas that can be spent for this transaction
   </td>
  </tr>
  <tr>
   <td>gas_unit_price
   </td>
   <td>u64
   </td>
   <td>Maximum gas price to be paid per unit of gas
   </td>
  </tr>
  <tr>
   <td>expiration_time
   </td>
   <td>u64
   </td>
   <td>The expiration time (Unix Epoch in seconds) for this transaction
   </td>
  </tr>
  <tr>
   <td>script_hash
   </td>
   <td>string
   </td>
   <td>Hex-encoded hash of the script used in this transaction
   </td>
  </tr>
  <tr>
   <td>script
   </td>
   <td>Object
   </td>
   <td>The transaction script and arguments of this transaction, represented as one of <a href="#PeerToPeerScript---type">PeerToPeerScript</a>, <a href="#MintScript---type">MintScript</a> or <a href="#UnknownScript---type">UnknownScript</a>.
   </td>
  </tr>
</table>



## UnknownTransaction - type

**Description**

Metadata for unsupported transaction types


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>type
   </td>
   <td>string
   </td>
   <td>Const string “unknown”
   </td>
  </tr>
</table>




---



## PeerToPeerTransferScript - type

**Description**

Transaction script for peer-to-peer transfer of resource


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>type
   </td>
   <td>string
   </td>
   <td>Const string “peer_to_peer_transaction”
   </td>
  </tr>
  <tr>
   <td>receiver
   </td>
   <td>Hex string
   </td>
   <td>The receiver libra address
   </td>
  </tr>
  <tr>
   <td>auth_key_prefix
   </td>
   <td>Hex string
   </td>
   <td>The auth_key_prefix
   </td>
  </tr>
  <tr>
   <td>amount
   </td>
   <td>u64
   </td>
   <td>The amount of microlibras being sent
   </td>
  </tr>
  <tr>
   <td>metadata
   </td>
   <td>Hex string
   </td>
   <td>The metadata supplied
   </td>
  </tr>
</table>



## MintScript - type

**Description**

Transaction script for a special transaction used by the faucet to mint Libra and send to a user.


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>type
   </td>
   <td>string
   </td>
   <td>Const string “mint_transaction”
   </td>
  </tr>
  <tr>
   <td>receiver
   </td>
   <td>Hex string
   </td>
   <td>The receiver libra address
   </td>
  </tr>
  <tr>
   <td>auth_key_prefix
   </td>
   <td>Hex string
   </td>
   <td>The auth_key_prefix
   </td>
  </tr>
  <tr>
   <td>amount
   </td>
   <td>u64
   </td>
   <td>The amount of microlibras being sent
   </td>
  </tr>
</table>




---



## UnknownScript - type

Description

Currently unsupported transaction script


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>type
   </td>
   <td>string
   </td>
   <td>Const string “unknown_transaction”
   </td>
  </tr>
</table>




---



## Event - type

**Description**

An event emitted during a transaction


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>key</strong>
   </td>
   <td>string
   </td>
   <td>Gobally unique identifier of event stream
   </td>
  </tr>
  <tr>
   <td><strong>sequence_number</strong>
   </td>
   <td>integer
   </td>
   <td>Sequence number of the current event in the given even stream
   </td>
  </tr>
  <tr>
   <td><strong>transaction_version</strong>
   </td>
   <td>integer
   </td>
   <td>Version of the transaction that emitted this event
   </td>
  </tr>
  <tr>
   <td><strong>data</strong>
   </td>
   <td>object
   </td>
   <td><a href="#ReceivedPaymentEvent---type">ReceivedPayment</a> or <a href="#SentPaymentEvent---type">SentPayment</a> or <a href="#UnknownEvent---type">UnknownEvent</a> object
   </td>
  </tr>
</table>



##

---



## ReceivedPaymentEvent - type

**Description**

Event emitted when an account received a payment.


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>type</strong>
   </td>
   <td>string
   </td>
   <td>Const string “receivedpayment”
   </td>
  </tr>
  <tr>
   <td><strong>amount</strong>
   </td>
   <td><a href="#amount---type">Amount</a>
   </td>
   <td>Amount received from the sender of the transaction
   </td>
  </tr>
  <tr>
   <td><strong>sender</strong>
   </td>
   <td>Hex string
   </td>
   <td>Hex-encoded address of the sender of the transaction that emitted the given event
   </td>
  </tr>
  <tr>
   <td><strong>metadata</strong>
   </td>
   <td>Hex string
   </td>
   <td>An optional field that can contain extra metadata for the event.
<p>
Note: This information can be used by an off-chain API to implement a sub-addressing scheme for a wallet.
   </td>
  </tr>
</table>




---



## SentPaymentEvent - type

**Description**

Event emitted when an account sends a payment.


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>type</strong>
   </td>
   <td>string
   </td>
   <td>Const string “sentpayment”
   </td>
  </tr>
  <tr>
   <td><strong>amount</strong>
   </td>
   <td><a href="#amount---type">Amount</a>
   </td>
   <td>Amount transferred in a transaction
   </td>
  </tr>
  <tr>
   <td><strong>receiver</strong>
   </td>
   <td>string
   </td>
   <td>Hex-encoded address of the receiver of an associated transaction
   </td>
  </tr>
  <tr>
   <td><strong>metadata</strong>
   </td>
   <td>string
   </td>
   <td>An optional field that can contain extra metadata for the event.
<p>
Note: This information can be used by another API to implement a subaddressing scheme for a wallet
   </td>
  </tr>
</table>




---



## UnknownEvent - type

**Description**

Represents events currently unsupported by JSON-RPC API.


### Attributes


<table>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><strong>type</strong>
   </td>
   <td>string
   </td>
   <td>Const string “unknown”
   </td>
  </tr>
</table>
