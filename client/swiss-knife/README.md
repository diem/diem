# Diem Swiss Knife

`swiss-knife` can be used to generate and serialize (BCS) raw and signed Diem transactions for a supported set of move scripts (See the `MoveScriptParams` enum for a list of supported MoveScripts). This binary is intended for use both by humans (interactive use) and to be called by automation (programs in any language which support json).

`swiss-knife` expects json input from stdin (wherever applicable) and writes json output to stdout. The output json object will always have two fields: `error_message` and `data`. If the operation succeeds, then `data` is set, otherwise `error_message` is set. Only one of them will be set and the other one will be an empty string.

Typical invocation looks like `swiss-knife operation_name < operation_specific_input.json`. This writes the output json to stdout. Run `cargo run -p swiss-knife -- --help` to print all the supported operations.

The `sample_inputs` folder contains a list of sample json inputs for various operations

## Building the binary in a release optimized mode

1. Run this from the `swiss-knife` directory : `cargo build --release`
2. The built binary can be found in `<repo-root>/target/release/swiss-knife`

## Examples for generate-raw-txn and generate-signed-txn operations

```
# Generate a peer_to_peer_transafer raw transaction (https://github.com/diem/diem/blob/1f86705b/language/transaction-builder/src/generated.rs#L447)
# For the txn_params json schema, look at the `struct TxnParams`
# For the script_params json schema, look at the `struct MoveScriptParams`
# Note about chain_id: For chain_id parameter, refer to the `enum NamedChain` in `chain_id.rs`. Also, please note that the numeric representation of the chain id ("0", "1", "2", etc.) can also be passed to the chain_id

$ cargo run -p swiss-knife -- generate-raw-txn < sample_inputs/generate_raw_txn_peer_to_peer_transfer.json

{
  "error_message": "",
  "data": {
    "raw_txn": "e1b3d22871989e9fd9dc6814b2f4fc412a0000000000000001e101a11ceb0b010000000701000202020403061004160205181d0735610896011000000001010000020001000003020301010004010300010501060c0108000506080005030a020a020005060c05030a020a020109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479087061795f66726f6d1b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010c0b0011000c050e050a010a020b030b0438000b05110202010700000000000000000000000000000001034c4252034c425200040371e931795d23e9634fd24a5992065f6b0164000000000000000400040040420f00000000000000000000000000034c4252fc24f65e0000000004",
    "script": "01e101a11ceb0b010000000701000202020403061004160205181d0735610896011000000001010000020001000003020301010004010300010501060c0108000506080005030a020a020005060c05030a020a020109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479087061795f66726f6d1b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010c0b0011000c050e050a010a020b030b0438000b05110202010700000000000000000000000000000001034c4252034c425200040371e931795d23e9634fd24a5992065f6b01640000000000000004000400"
  }
}

# Generate a signed transaction from the previous raw transaction and signature
$ cargo run -p swiss-knife -- generate-signed-txn < sample_inputs/generate_signed_txn_peer_to_peer_transfer.json

{
  "error_message": "",
  "data": {
    "raw_txn": "e1b3d22871989e9fd9dc6814b2f4fc412a0000000000000001d401a11ceb0b0100000007010002020204030610041602051815072d60088d011000000001010000020001000003020301010004010300010501060c01080003060c060800030002060c030109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479077072656275726e1b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010a0a0011000c020b000e020a0138000b02110202010700000000000000000000000000000001034c4252034c4252000101640000000000000040420f00000000000000000000000000034c4252fc24f65e0000000004",
    "script": "01d401a11ceb0b0100000007010002020204030610041602051815072d60088d011000000001010000020001000003020301010004010300010501060c01080003060c060800030002060c030109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479077072656275726e1b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010a0a0011000c020b000e020a0138000b02110202010700000000000000000000000000000001034c4252034c42520001016400000000000000"
  }
}

# Generate a preburn raw transaction (https://github.com/diem/diem/blob/1f86705b/language/transaction-builder/src/generated.rs#L480)
$ cargo run -p swiss-knife -- generate-raw-txn < sample_inputs/generate_raw_txn_preburn.json

{
  "error_message": "",
  "data": {
    "raw_txn": "e1b3d22871989e9fd9dc6814b2f4fc412a0000000000000001d401a11ceb0b0100000007010002020204030610041602051815072d60088d011000000001010000020001000003020301010004010300010501060c01080003060c060800030002060c030109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479077072656275726e1b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010a0a0011000c020b000e020a0138000b02110202010700000000000000000000000000000001034c4252034c4252000101640000000000000040420f00000000000000000000000000034c4252fc24f65e0000000004"
  }
}

# Generate a rotate_dual_attestation_info raw transaction
$ cargo run -p swiss-knife -- generate-raw-txn < sample_inputs/generate_raw_txn_rotate_dual_attestation_info.json

{
  "error_message": "",
  "data": {
    "raw_txn": "e1b3d22871989e9fd9dc6814b2f4fc412a00000000000000018f01a11ceb0b010000000501000203020a050c0d07193d08561000000001000100000200010002060c0a020003060c0a020a020f4475616c4174746573746174696f6e0f726f746174655f626173655f75726c1c726f746174655f636f6d706c69616e63655f7075626c69635f6b657900000000000000000000000000000001000201070a000b0111000b000b021101020002041c68747470733a2f2f6578616d706c652e636f6d2f656e64706f696e740420edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e7703340420f00000000000000000000000000034c4252fc24f65e0000000004",
    "script": "018f01a11ceb0b010000000501000203020a050c0d07193d08561000000001000100000200010002060c0a020003060c0a020a020f4475616c4174746573746174696f6e0f726f746174655f626173655f75726c1c726f746174655f636f6d706c69616e63655f7075626c69635f6b657900000000000000000000000000000001000201070a000b0111000b000b021101020002041c68747470733a2f2f6578616d706c652e636f6d2f656e64706f696e740420edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e77033"
  }
}
```

# Helper operations for testing

## Generate a Ed25519 Keypair

Generate a Ed25519 Keypair and corresponding diem account address and auth key associated with this keypair.
Optionally provide a seed to deterministically generate a Ed25519 Keypair


```shell script
# Generate a random keypair
$ cargo run -p swiss-knife -- generate-test-ed25519-keypair

{
  "error_message": "",
  "data": {
    "diem_account_address": "e1b3d22871989e9fd9dc6814b2f4fc41",
    "diem_auth_key": "5a06116a9801533249b06eeef54db2f1e1b3d22871989e9fd9dc6814b2f4fc41",
    "private_key": "b2f7f581d6de3c06a822fd6e7e8265fbc00f8401696a5bdc34f5a6d2ff3f922f",
    "public_key": "edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e77033"
  }
}

# Generate a deterministic keypair with the provided u64 seed
$ cargo run -p swiss-knife -- generate-test-ed25519-keypair --seed 0

{
  "error_message": "",
  "data": {
    "diem_account_address": "e1b3d22871989e9fd9dc6814b2f4fc41",
    "diem_auth_key": "5a06116a9801533249b06eeef54db2f1e1b3d22871989e9fd9dc6814b2f4fc41",
    "private_key": "b2f7f581d6de3c06a822fd6e7e8265fbc00f8401696a5bdc34f5a6d2ff3f922f",
    "public_key": "edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e77033"
  }
}
```

## Sign a transaction using a Ed25519 Private Key

```shell script
$ cargo run -p swiss-knife -- sign-transaction-using-ed25519 < sample_inputs/generated_raw_txn_p2p_transfer.json

{
  "error_message": "",
  "data": {
    "signature": "193eabce444d5cca25bb18591a2dca11688a2cc513852bca52016cab309be67a4fd409f1c9c162a7f2ab9265faeb22eb6e03a52196592d3bf7f96195ce08ae08"
  }
}
```

## Verify a transaction signature using a Ed25519 Public Key

```shell script
$ cargo run -p swiss-knife -- verify-transaction-ed25519-signature < sample_inputs/generated_signed_txn_p2p_transfer.json

{
  "error_message": "",
  "data": {
    "valid_signature": true
  }
}
```


## Verify a payload signature using a Ed25519 Public Key

```shell script
$ cargo run -p swiss-knife -- verify-ed25519-signature < sample_inputs/verify_signature_payload.json

{
  "error_message": "",
  "data": {
    "valid_signature": true
  }
}
```

(Note that this is not a test of a transaction signature. This only tests the
"raw" internal Ed255519 library used in Diem, and will only allow you to
e.g. get certainty that we use Ed25519 variant 'pure' as opposed to
'pre-hashed'. For testing a transaction signature, see the previous example)
