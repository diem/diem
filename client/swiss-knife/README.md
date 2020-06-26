# Libra Swiss Knife

`swiss-knife` can be used to generate and serialize (LCS) raw and signed Libra transactions for a supported set of move scripts (See the `MoveScriptParams` enum for a list of supported MoveScripts). This binary is intended for use both by humans (interactive use) and to be called by automation (programs in any language which support json).

`swiss-knife` expects json input from stdin (wherever applicable) and writes json output to stdout. The output json object will always have two fields: `error_message` and `data`. If the operation succeeds, then `data` is set, otherwise `error_message` is set. Only one of them will be set and the other one will be an empty string.

Typical invocation looks like `swiss-knife operation_name < operation_specific_input.json`. This writes the output json to stdout. Run `cargo run --package swiss-knife -- --help` to print all the supported operations.

The `sample_inputs` folder contains a list of sample json inputs for various operations

## Building the binary in a release optimized mode

1. Run this from the `swiss-knife` directory : `cargo build --release`
2. The built binary can be found in `<repo-root>/target/release/swiss-knife`

## Examples for generate-raw-txn and generate-signed-txn operations

```
# Generate a peer_to_peer_transafer raw transaction (https://github.com/libra/libra/blob/1f86705b/language/transaction-builder/src/generated.rs#L447)
# For the txn_params json schema, look at the `struct TxnParams`
# For the script_params json schema, look at the `struct MoveScriptParams`
$ cargo run --package swiss-knife -- generate-raw-txn < sample_inputs/generate_raw_txn_peer_to_peer_transfer.json
{
  "error_message": "",
  "data": {
    "raw_txn": "e1b3d22871989e9fd9dc6814b2f4fc412a0000000000000001ed01a11ceb0b01000701000202020403061004160205181d07356f08a4011000000001010000020001000003020301010004010300010501060c0108000506080005030a020a020005060c05030a020a020109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479167061795f66726f6d5f776974685f6d657461646174611b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010c0b0011000c050e050a010a020b030b0438000b05110202010700000000000000000000000000000001034c4252034c425200040371e931795d23e9634fd24a5992065f6b0164000000000000000400040040420f00000000000000000000000000034c4252fc24f65e00000000",
    "raw_txn_hash": "a293a4b658d76bea6c07f3cf064b243f7a4f1087054ce1072029c148d1ab3a42"
  }
}

# Generate a signed transaction from the previous raw transaction and signature
$ cargo run --package swiss-knife -- generate-signed-txn < sample_inputs/generate_signed_txn_peer_to_peer_transfer.json

{
  "error_message": "",
  "data": {
    "signed_txn": "e1b3d22871989e9fd9dc6814b2f4fc412a0000000000000001ed01a11ceb0b01000701000202020403061004160205181d07356f08a4011000000001010000020001000003020301010004010300010501060c0108000506080005030a020a020005060c05030a020a020109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479167061795f66726f6d5f776974685f6d657461646174611b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010c0b0011000c050e050a010a020b030b0438000b05110202010700000000000000000000000000000001034c4252034c425200040371e931795d23e9634fd24a5992065f6b0164000000000000000400040000000000000000000000000000000000034c42526922f45e000000000020edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e7703340439b76391ca951078d86d5e916288ecbc7bea5e5c79a869ad1f5842b8b3452940820ffbfb53f8b209166b07051adbafed13a38dd322476175f7823293c834401"
  }
}

# Generate a preburn raw transaction (https://github.com/libra/libra/blob/1f86705b/language/transaction-builder/src/generated.rs#L480)
$ cargo run --package swiss-knife -- generate-raw-txn < sample_inputs/generate_raw_txn_preburn.json
{
  "error_message": "",
  "data": {
    "raw_txn": "e1b3d22871989e9fd9dc6814b2f4fc412a00000000000000018602a11ceb0b010007010004020409030d1604230405272107487708bf011000000001000001010101030100000200010101010402030001050301000106040501010307000702060c0b000109000001060c0108010206080103010b0001090002060c03010900054c696272610c4c696272614163636f756e740a7072656275726e5f746f1257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c6974791b726573746f72655f77697468647261775f6361706162696c6974790d77697468647261775f66726f6d00000000000000000000000000000001010106030b0a0011010c020b000e020a01380038010b02110202010700000000000000000000000000000001034c4252034c4252000101640000000000000040420f00000000000000000000000000034c4252fc24f65e00000000",
    "raw_txn_hash": "5f1256d71f183f15de24ad80c96f77919c8c1ff97cab0727d10f7b8367a9b1ec"
  }
}

# Generate a signed transaction from the previous raw transaction and signature
$ cargo run --package swiss-knife -- generate-signed-txn < sample_inputs/generate_signed_txn_preburn.json

{
  "error_message": "",
  "data": {
    "signed_txn": "0652ce646d18acd435b1817ff47560d12a00000000000000018602a11ceb0b010007010004020409030d1604230405272107487708bf011000000001000001010101030100000200010101010402030001050301000106040501010307000702060c0b000109000001060c0108010206080103010b0001090002060c03010900054c696272610c4c696272614163636f756e740a7072656275726e5f746f1257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c6974791b726573746f72655f77697468647261775f6361706162696c6974790d77697468647261775f66726f6d00000000000000000000000000000001010106030b0a0011010c020b000e020a01380038010b02110202010700000000000000000000000000000001034c4252034c4252000101640000000000000000000000000000000000000000000000034c42522422f45e000000000020edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e7703340e6bd4ec1853c375338613f8ca9dcb9c01fd77c59736dc14e042c3bb55b7c050d75507a766a667341aaa705551044927e3bbaa519d18ddc66163adf4265769e02"
  }
}
```

# Helper operations for testing

## Generate a Ed25519 Keypair

Generate a Ed25519 Keypair and corresponding libra account address and auth key associated with this keypair. seed is an unsigned 64 bit integer.


```shell script
$ cargo run --package swiss-knife -- generate-test-ed25519-keypair --seed 0

{
  "error_message": "",
  "data": {
    "libra_account_address": "e1b3d22871989e9fd9dc6814b2f4fc41",
    "libra_auth_key": "5a06116a9801533249b06eeef54db2f1e1b3d22871989e9fd9dc6814b2f4fc41",
    "private_key": "b2f7f581d6de3c06a822fd6e7e8265fbc00f8401696a5bdc34f5a6d2ff3f922f",
    "public_key": "edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e77033"
  }
}
```

## Sign a payload using a Ed25519 Private Key

```shell script
$ cargo run --package swiss-knife -- sign-payload-using-ed25519 < sample_inputs/sign_payload.json

{
  "error_message": "",
  "data": {
    "signature": "babc63f10356a4ed4d56d5da2f022cd3683fb41a56aed2cd764c276fb9f9339e7b2c1cfe337d9b0fb54fb6932180f621f2494d0bf7fdb0a2aa1e06ba00682e00"
  }
}
```

## Verify a payload signature using a Ed25519 Public Key

```shell script
$ cargo run --package swiss-knife -- verify-ed25519-signature < sample_inputs/verify_signature_payload.json

{
  "error_message": "",
  "data": {
    "valid_signature": true
  }
}
```
