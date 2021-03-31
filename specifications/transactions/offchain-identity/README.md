# Off-Chain Identity

Diem Payment Network wallets benefit from consistent standards for naming and addressing across participants and their clients.

## Subaddresses

Each account on-chain is represented by a 16-byte value called an **account address**. To allow multiplexing of a single address into distinct off-chain identities, or wallets, a participant may use a **subaddress**. For convenience, Diem defines a standard format for representing the combination of the an account address and a subaddress as an **account identifier**. Account identifiers have a dedicated URI representation including parameters called an **intent identifier**. Subaddresses, account identifiers, and intent identifiers are defined in [DIP-5](https://dip.diem.com/dip-5/).

### Subaddresses

- 8 bytes
- Unambiguous (i.e., Recipient VASPs should not issue the same subaddress to multiple clients)
- The zero subaddress (`0x0000000000000000`) is reserved to denote the root (VASP owned) account.
- Subaddresses should be single-use and should not be generated from personal information, such as names, email addresses, or government-issued identification numbers.

### Account Identifiers

Account Identifier format: `<prefix> | <delimiter> | <version> | <encoded payload> | <checksum>`

- A prefix (also known as hrp (human readable part) which identifies the network version this address is intended for
  - “dm” for Mainnet addresses
  - “tdm” for Testnet addresses
  - "pdm" for Pre-Mainnet addresses
- A Bech32 delimiter: The character “1” (one)
- A Bech32 version identifier: The character “p” (version = 1) for on-chain with subaddress
- A Bech32 encoded payload: For version 1, is Diem account address + subaddress (16 + 8 bytes)
- The last 6 characters correspond to the Bech32 checksum

The Diem Account Identifier must not be mixed-cases and represented by only all upper or lower case.  For example, both `dm1pptdxvfjck4jyw3rkfnm2mnd2t5qqqqqqqqqqqqq305frg` or `DM1PPTDXVFJCK4JYW3RKFNM2MND2T5QQQQQQQQQQQQQ305FRG` are valid but `dm1pptdXVFJCK4JYW3RKFNM2MND2t5qqqqqqqqqqqqq305frg` is not.


#### Example with explicit subaddress
Identifier information
* Prefix (string)
  * Network: `dm`
* Address type (version prefix): `01` (letter p in Bech32 alphabet)
* Address payload (in hex)
  * Address: `0xf72589b71ff4f8d139674a3f7369c69b`
  * Subaddress: `0xcf64428bdeb62af2`
* Checksum: `2vfufk`

**Account identifier**: `dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk`

#### Example without explicit subaddress (root account)
Identifier information
* Prefix (string)
  * Network: `dm`
* Address type (version prefix): `01` (letter p in Bech32 alphabet)
* Address payload (in hex)
  * Address: `0xf72589b71ff4f8d139674a3f7369c69b`
  * Subaddress: `0x0000000000000000`
* Checksum: `d8p9cq`

**Account identifier**: `dm1p7ujcndcl7nudzwt8fglhx6wxnvqqqqqqqqqqqqqd8p9cq`

### Intent identifiers

The Intent Identifier consists of
* A prefix: `diem://` to explicitly specify how the identifier should be interpreted
* A base URI: *account identifier*
* Query parameters
  * Provides details for how the request should be fulfilled
  * Currently supported parameters:
    * currency uses the 'c' key, with the value encoded as a 3-letter currency code (ex. XDX)
    * amount uses the 'am' key, with the value encoded in micro units (10e-6)

#### Example: Request Intent

This intent represents a request to receive funds at a given address. Since neither the amount nor currency are prefilled, the sender will define these fields.

* Prefix: `diem://`
* Base URI: Account Identifier: `dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk`
* Query parameters: none

**Intent identifier**: `diem://dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk`

#### Example:  Request Intent with Currency
This intent represents a request to receive funds in a specific currency at a given address. The amount is defined by the sender.

* Prefix: `diem://`
* Base URI: Account Identifier: `dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk`
* Query parameters: `c=XDX` (currency preference is XDX composite token)

**Intent identifier**: `diem://dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk?c=XDX`

### Example: Request Intent with Currency and Amount
This intent represents a request to receive a specific amount in a specific currency for a given address.

Identifier information
* Prefix: `diem://`
* Base URI: Account Identifier: `dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk`
* Query parameters
  * `c=XDX` (currency preference is XDX composite token)
  * `am=1000000` (for 1 XDX, amount is 1000000 µ-units)

**Intent identifier**: `diem://dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk?c=XDX&am=1000000`
