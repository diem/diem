---
title: "Accounts"
id: "basics-accounts"
hidden: false
---
An account represents a resource on the Diem Blockchain that can send transactions. Each account is identified by a particular 16-byte account address and is a container for Move modules and Move resources.

## Introduction

The state of each account comprises both code and data:

- **Code**: Move modules contain code (type and procedure declarations), but they do not contain data. The module's procedures encode the rules for updating the blockchain's global state.
- **Data**: Move resources contain data but no code. Every resource value has a type that is declared in a module published in the blockchain's distributed database.

An account may contain an arbitrary number of Move resources and Move modules. The Diem Payment Network (DPN) supports accounts created for Regulated Virtual Asset Service Providers [Regulated VASP](../reference/glossary#regulated-vasp) and Designated Dealers.

## Account address

A Diem account address is a 16-byte value. The account address is derived from a cryptographic hash of its public verification key concatenated with a signature scheme identifier byte. The Diem Blockchain supports two signature schemes:[Ed25519](../reference/glossary#ed25519) and MultiEd25519 (for multi-signature transactions). You will need the account's private key to sign a transaction.

An account address is derived from its initial authentication key.

### Generate an auth key and account address
Each account stores an authentication key used to authenticate the signer of a transaction. The DPN supports rotating the auth key of an account without changing its address. This means that the account's initial auth key is replaced by another newly generated auth key.

To generate an authentication key and account address:

1. **Generate a key pair**: Generate a fresh key-pair (pubkey_A, privkey_A). The DPN uses the PureEdDSA scheme over the Ed25519 curve, as defined in RFC 8032.
2. **Derive a 32-byte authentication key**: Derive a 32-byte authentication key `auth_key = sha3-256(K_pub | 0x00)`, where | denotes concatenation. The 0x00 is a 1-byte signature scheme identifier where 0x00 means single-signature. The first 16 bytes of `auth_key` is the “auth key prefix”. The last 16 bytes of `auth_key` is the account address. Any transaction that creates an account needs both an account address and an auth key prefix, but a transaction that is interacting with an existing account only needs the address.

#### Multisig authentication
The authentication key for an account may require either a single signature or multiple signatures ("multisig"). With K-of-N multisig authentication, there are a total of N signers for the account, and at least K of those N signatures must be used to authenticate a transaction.

Creating a K-of-N multisig authentication key is similar to creating a single signature one:
1. **Generate key pairs**: Generate N ed25519 public keys p_1, …, p_n.
2. **Derive a 32-byte authentication key**: Compute `auth_key = sha3-256(p_1 | … | p_n | K | 0x01)`. Derive an address and an auth key prefix as described above. The 0x01 is a 1-byte signature scheme identifier where 0x01 means multisignature.

## Account resources

Every account on the DPN is created with at least two resources:

* [RoleId](https://github.com/diem/diem/blob/main/language/diem-framework/modules/doc/Roles.md#resource-roleid), which grants the account a single, immutable [role](basics-accounts.md#account-roles) for [access control](https://github.com/diem/dip/blob/main/dips/dip-2.md).
* [DiemAccount](https://github.com/diem/diem/blob/main/language/diem-framework/modules/doc/DiemAccount.md#resource-diemaccount), which holds the account’s [sequence number](../reference/glossary#sequence-number), authentication key, and event handles.

### Currencies

The DPN supports an account transacting in different currencies.

From a standards perspective, [`Diem<CoinType>`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/doc/Diem.md#resource-diem) is the Diem Blockchain equivalent of [ERC20](https://eips.ethereum.org/EIPS/eip-20). At the Move level, these are different generic instantiations of the same Diem resource type (e.g., `Diem<Coin1>`, `Diem<XUS>`).

`Diem<XUS>` will be the currency type available at launch.

### Balances

A zero balance of `Diem<CoinType>` is added whenever `Diem<CoinType>` currency is authorized for an account.

Each non-administrative account stores one or more [Balance`<CoinType>`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/doc/DiemAccount.md#resource-balance) resources. For each currency type that the account holds such as `Diem<Coin1>` or `Diem<XUS>`, there will be a separate Balance resource such as Balance`<Diem<Coin1>>` or Balance`<Diem<XUS>>`.

When a client sends funds of type CoinType to an account, they should:
* check if the account address exists
* ensure that the account address has a balance in CoinType, even if that balance is zero.

To send and receive `Diem<CoinType>`, an account must have a balance in `Diem<CoinType>`. A transaction that sends `Diem<CoinType>` to an account without a balance in `Diem<CoinType>` will abort.

Balances can be added either at account creation or subsequently via the [add_currency script](../transactions/txns-types/txns-manage-accounts.md#add-a-currency-to-an-account). Only the account owner can add new balances after account creation. Once you add a balance to an account, it cannot be removed. For example, an account that accepts `Diem<XUS>` will always accept `Diem<XUS>`.

## Account roles

All Regulated VASPs can have two kinds of accounts, each with a different role -- ParentVASP and ChildVASP accounts.

### ParentVASP
Each Regulated VASP has one unique root account called ParentVASP. A ParentVASP carries three key pieces of data - its name, the endpoint URL to hit for off-chain APIs, and a compliance public key for authenticating signatures on off-chain data payloads.

### ChildVASP
ChildVASP is a child account of a particular ParentVASP. A Regulated VASP need not have any child accounts, but child accounts allow a Regulated VASP to maintain a structured on-chain presence if it wishes (e.g., separate cold/warm/hot accounts).

A ChildVASP knows the address of its ParentVASP. If off-chain communication is required when transacting with a ChildVASP, clients should use this address to look up the ParentVASP information.


## Create accounts

When the Diem main network (mainnet) is launched, only the [TreasuryCompliance account](https://github.com/diem/dip/blob/main/dips/dip-2.md#roles) can create ParentVASP accounts. Once a ParentVASP account is created, the Regulated VASP can then create ChildVASP accounts.

To create a new account, the creator must specify
* the address of the new account
* its authentication key prefix, and
* the currencies that the account will initially accept.

You can only send funds to an address that already contains an account. If you send funds to an empty address, no account will be created for that address and the create account transaction will abort.

Learn more about how accounts are created [here](../transactions/txns-types/txns-create-accounts-mint.md).
