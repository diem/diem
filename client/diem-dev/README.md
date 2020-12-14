# Exercising New Diem Functionality in Testnet

Diem has changed a lot since 6/18 and the new whitepaper, but I don't think we've written anything comprehensive about how to exercise all of this new functionality. The linked document attempts to explain the state of the world in Diem master at a level of abstraction appropriate for any client that wants to hit testnet, not a specific client (that is, there will be no Diem CLI or json-rpc snippets here).

## Overview of New Functionality

### VASP Account Roles

In the original Diem design, all accounts were the same. Now, every account has exactly one *role*. There are seven roles in all (`ParentVASP`, `ChildVASP`, `DesignatedDealer`, `Validator`, `ValidatorOperator`, `TreasuryCompliance`, and `AssocRoot`), but we will focus on the client-relevant `ParentVASP` and `ChildVASP` roles here. A VASP (virtual asset service provider) is a regulated wallet.

* A [ParentVASP](https://github.com/diem/diem/blob/master/language/stdlib/modules/vasp.move#L20;L34) is the unique root account for a particular VASP (i.e., there will be one account of this type per company). A `ParentVASP` carries three key pieces of data: its `human_name`, `base_url` (a URL containing an endpoint to hit for off-chain APIs like exchanging KYC information and the travel rule protocol), and a `compliance_public_key` (ed5519 public key for authenticating signatures on KYC and travel rule payloads).
* A [ChildVASP](https://github.com/diem/diem/blob/master/language/stdlib/modules/vasp.move#L40) is a child account of a particular parent VASP. A VASP need not have any child accounts, but child accounts allow more sophisticated a VASP to maintain a structured on-chain presence (e.g., separate cold/warm/hot accounts). A parent VASP can currently have an unbounded number of children (though we'll need to pick a reasonable [limit](https://github.com/diem/diem/issues/3949)). A child knows the account address of its parent VASP. When transacting with a child account, clients should use this to look up its parent, then use the `base_url` and `compliance_public_key` of the parent for off-chain communication.

### Off-Chain Protocols

As mentioned above, some transactions between VASPs must use off-chain protocols for exchanging information. These APIs are implemented by VASPs themselves, not by Diem testnet (though there is an in-progress effort to provide a reference endpoint for testing). However, it's easy enough to mimic the relevant aspects of these protocols for testnet purposes (more on this later).

### Currencies

There are two currencies in testnet: `XUS` (stablecoin), and `XDX` (synthetic currency that is currently a stub and will be updated once the final composition is determined). At the Move level, these are different generic instantiation of the same `Diem` type (i.e.`Diem<XUS>`, `Diem<XDX>`).

### Addresses, Authentication Keys, and Cryptographic Keys

Two major changes in new Diem design are the [shrinking](https://github.com/diem/diem/issues/2764) of account addresses from 32 bytes to 16 bytes and the addition of multisignature authentication keys. A quick primer on how these concepts fit together, since they are very important for clients:

* To create a fresh account address, generate an ed25519 keypair `(K_pub, K_priv)`.
* Derive a 32 byte authentication key `auth_key = sha3-256(K_pub | 0)`. The `0` is a signature scheme identifier where `0` means single-signature and `1` means multisig.
* Each account has an `auth_key` that is checked against the public key included in the transaction in order to authenticate the sender.
* The account address is the last 16 bytes of `auth_key`
* The first 16 bytes of `auth_key` is the “auth key prefix”. Any transaction that creates an account needs both an account address and an auth key prefix, but a transaction that is sending funds to an existing account needs only the address.
* Creating a `K`-of-`N` [multisig](https://github.com/diem/diem/blob/master/crypto/crypto/src/multi_ed25519.rs) authentication key is similar: generate `N` ed25519 public keys, then compute `auth_key = (<concatenated_keys> | K | 1)`. Derive an address and an auth key prefix as described above.
* Diem supports key rotation via changing the `auth_key` stored under an account. The address associated with an account never changes.

## Exercising New Functionality in Testnet

### Creating accounts and minting with the faucet

As in the original Diem, testnet has a faucet service for creating accounts and giving money to existing accounts. There are three important changes from the original faucet:

* The faucet now [needs](https://github.com/diem/diem/pull/3972) one of the currency codes above
* The faucet can accept either a 16 byte address (for an existing account), a 32 byte account authentication key (for an account that does not yet exist and should be created by the faucet). Previously, account addresses and authentication keys were both 32 byte values, but account addresses have [shrunk](https://github.com/diem/diem/issues/2764).
* An account created by the faucet has the `ParentVASP` role. Note that in mainnet, only the Association will have the privilege to create `ParentVASP`s, and will do so only for entities that meet appropriate standards.

### Transactions

Transactions are mostly unchanged from the original design. Two notable differences are:

* Transactions now require a [`gas_currency_code`](https://github.com/diem/diem/blob/master/types/src/transaction/mod.rs#L72) specifying which currency should be used to pay for gas. The account must have a balance in this currency.
* The signature scheme for a transaction can be either be single-signature ed25519 (as the original design) or multi-ed25519, a new [multisig](https://github.com/diem/diem/issues/2431) format

### Payments

Payments can be sent using [this](https://github.com/diem/diem/blob/master/language/stdlib/transaction_scripts/peer_to_peer_with_metadata.move) transaction script. There are a few changes:

* The script requires a generic type parameter `Token` specifying the currency to be transferred. The sending and receiving account must both have a balance in this currency. A transaction that attempts to send (e.g.) `XDX` to an account that doesn’t have an `XDX` balance will abort.
* The script has a `metadata` parameter that accepts arbitrary binary data. For most transactions, the metadata should be the subaddress of the VASP customer receiving the payment. The contents of `metadata` are emitted in payment events, but are not otherwise inspected on-chain (so using empty or dummy `metadata` is just fine for testing).
* The script has a `metadata_signature` parameter used for dual attestation in the travel rule protocol (see below).
* Sending a payment to an address that does not exist will abort (*not* create a new account at that address).

### Creating Child VASP Accounts

As mentioned above, any account created by the faucet has the `ParentVASP` role. You can create a child VASP account from a parent VASP using the [`create_child_vasp_account`](https://github.com/diem/diem/blob/master/language/stdlib/transaction_scripts/create_child_vasp_account.move) transaction script. Attempting to create a child VASP account from another child VASP will abort.

### Adding Currencies to Accounts

Each newly created account accepts at least one currency, which as specified as a currency code type parameter to the account creation script. New currencies can be added to an existing account via the `add_currency_to_account` [script](https://github.com/diem/diem/blob/master/language/stdlib/transaction_scripts/add_currency_to_account.move). In addition, a child VASP account can be prepopulated with all currencies in the system by using the `add_all_currencies` flag in the script above.

### Dual Attestation/Travel Rule Protocol

In Diem mainnet every payment transaction between two **distinct **VASP accounts (parent <-> child and child <-> child within the same VASP are exempt) over a certain threshold (1000 XDX, for now) must perform dual attestation in order to comply with the travel rule. "Dual attestation" is a fancy way of saying:

* The payer must send the payee some data
* The payee must sign the data with its `compliance_public_key`
* The payer must include the signature and the data in a payment transaction, and the signature will be checked on-chain

Details of doing this exchange in production will be published soon, but we will explain how to mock the protocol locally for testing purposes.

Every parent VASP created by the faucet has a dummy `base_url` and  `compliance_public_key`. However, you can use [these](https://github.com/diem/diem/blob/master/language/stdlib/transaction_scripts/rotate_base_url.move) [scripts](https://github.com/diem/diem/blob/master/language/stdlib/transaction_scripts/rotate_compliance_public_key.move) to send a transaction from the parent VASP that sets the URL/key to meaningful values. Once this is done, you can construct the message to be signed and craft a dual attestation transaction with the following ingredients:

* `payer_vasp_address`: the address of the payer (can be either a parent or child VASP)
* `payee_vasp_address`: the address of the payee (can be either a parent or child VASP). Encoding: BCS `[u8; 16]`
* `amount`: number of coins to send. Encoding: BCS `u64`
* `reference_id`: an identifier for the payment in the off-chain protocol. This is not inspected on-chain, so dummy value suffices for testing.  Encoding: BCS `[u8;12]`

The payee VASP should sign the [BCS](https://docs.rs/bcs/)-encoded (see types above) message `reference_id | payer_vasp_address | amount | @@$$DIEM_ATTEST$$@@`. to produce a `payee_signature`. The `@@$$DIEM_ATTEST$$@@` part is a domain separator intended to  prevent misusing a  different signature from the same key (e.g., interpreting a KYC signature as a travel rule signature).

Finally, send a transaction from payer_vasp_address using the payment [script](https://github.com/diem/diem/blob/master/language/stdlib/transaction_scripts/peer_to_peer_with_metadata.move) mentioned above with `payee = payee_vasp_address, amount = amount, metadata = reference_id, metadata_signature = payee_signature`.
