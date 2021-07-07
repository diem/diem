# Faucet

Faucet is a service for creating and funding accounts on Diem Network by treasury compliance account.

It is created for Testnet usage only.

## Mint API

Mint API can create and fund your account. It will fund your account if it is already created onchain.

* Base URL: `http://faucet.testnet.diem.com/`
* Path: `/mint`
* Method: POST

URL Query Params:

| param name             | type   | required? |  description                                                         |
|------------------------|--------|-----------|----------------------------------------------------------------------|
| `amount`               | int    | Y         | amount of coins to mint                                              |
| `auth_key`             | string | Y         | your account authentication key                                      |
| `currency_code`        | string | Y         | the currency code, e.g. XDX                                          |
| `return_txns`          | bool   | N         | returns the transactions for creating / funding the account          |
| `is_designated_dealer` | bool   | N         | creates a designated dealer account instead of a parent VASP account |
| `vasp_domain` | string   | N         | domain for VASP to add or remove for parent VASP, is_designated_dealer must be set to false |
| `is_remove_domain` | bool   | N         | add or remove the above VASP domain to parent VASP account |

Notes:
* By default, the account created is a parent VASP account.
* Type bool means you set value to a string "true" or "false"
* For existing accounts as defined by the auth_key, the service submits 1 transfer funds transaction.
* For new accounts as defined by the auth_key, the service first issues a transaction for creating the account and another for transferring funds.
* All funds transferred come from the account 000000000000000000000000000000dd.
* Clients should retry their request if the requests or the transactions execution failed. One reason for failure is that, under load, the service may issue transactions with duplicate sequence numbers. Only one of those transactions will be executed, the rest will fail.

### Response

If no query param `return_txns` or it is not "true", server returns an unsigned int 64 in HTTP response body. The number is the account sequence number of the account `000000000000000000000000000000dd` on Testnet after executing the request.
Nominally, this number can be used for looking up the submitted transaction. However, under load, this number may be shared by multiple transactions.

Set query param `return_txns`, server will response all transactions for creating and funding your account.
The respond HTTP body is hex encoded bytes of BCS encoded `Vec<diem_types::transaction::SignedTransaction>`.

Decode Example ([source](https://diem.github.io/client-sdk-python/diem/testnet.html#diem.testnet.Faucet)):

``` python
  de = bcs.BcsDeserializer(bytes.fromhex(response.text))
  length = de.deserialize_len()

  txns = []
  for i in range(length):
    txns.push(de.deserialize_any(diem_types.SignedTransaction))

```

You should retry the mint API call if the returned transactions executed failed.


## Example

```bash
curl -X POST http://faucet.testnet.diem.com/mint\?amount\=1000000\&currency_code\=XUS\&auth_key\=459c77a38803bd53f3adee52703810e3a74fd7c46952c497e75afb0a7932586d\&return_txns\=true
01000000000000000000000000000000dd05a600000000000001e001a11ceb0b010000000701000202020403061004160205181d0735600895011000000001010000020001000003020301010004010300010501060c0108000506080005030a020a020005060c05030a020a020109000b4469656d4163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479087061795f66726f6d1b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010c0b0011000c050e050a010a020b030b0438000b051102020107000000000000000000000000000000010358555303585553000403a74fd7c46952c497e75afb0a7932586d0140420f00000000000400040040420f00000000000000000000000000035855532a610f6000000000020020056244e7bf776e471d818dc18fdf7b8833c5439ac9a96e126f8f32c7bc7c14b64026a2c45c8e4066c661dc4f36baa6ad61499999b548b9f63ad15853660c408cedec3078b7773a829ec48de8b04291cd11530734b2f91d5e42f35a4c6378cb7c09
```
