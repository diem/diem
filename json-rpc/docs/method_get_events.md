## Method get_events

**Description**

Fetch the events for a given event stream.


### Parameters


| Name           | Type           | Description                                                   |
|----------------|----------------|---------------------------------------------------------------|
| key            | string         | Globally unique identifier of an event stream                 |
| start          | unsigned int64 | The start of the event with this sequence number              |
| limit          | unsigned int64 | The maximum number of events retrieved                        |

Note:
1. For `sentpayment` and `receivedpayment` events, call [get_account](method_get_account.md) to get the event key of the event streams for a given user account.
2. For currency related events, call [get_currencies](method_get_currencies.md) to get the event keys (for example: to XDX exchange rate change event key).


### Returns

Returns array of [Event](type_event.md) objects


### Example


```
//Request: get events associated with receivedpayment event stream key "00000000000000001668f6be25668c1a17cd8caf6b8d2f25" for account "1668f6be25668c1a17cd8caf6b8d2f25"
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_events","params": ["00000000000000001668f6be25668c1a17cd8caf6b8d2f25", 0, 10], "id":1}' https://testnet.diem.com/v1

//Response
{
  "id": 1,
  "jsonrpc": "2.0",
  "diem_chain_id": 2,
  "diem_ledger_timestampusec": 1596694876315159,
  "diem_ledger_version": 3310435,
  "result": [
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 0,
      "transaction_version": 106495
    },
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 1,
      "transaction_version": 106564
    },
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 2,
      "transaction_version": 106608
    },
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 3,
      "transaction_version": 107186
    },
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 4,
      "transaction_version": 107271
    },
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 5,
      "transaction_version": 107333
    },
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 6,
      "transaction_version": 783134
    },
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 7,
      "transaction_version": 783282
    },
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 8,
      "transaction_version": 783378
    },
    {
      "data": {
        "amount": {
          "amount": 100000000,
          "currency": "XDX"
        },
        "metadata": "",
        "receiver": "1668f6be25668c1a17cd8caf6b8d2f25",
        "sender": "000000000000000000000000000000dd",
        "type": "receivedpayment"
      },
      "key": "00000000000000001668f6be25668c1a17cd8caf6b8d2f25",
      "sequence_number": 9,
      "transaction_version": 2371067
    }
  ]
}

```
