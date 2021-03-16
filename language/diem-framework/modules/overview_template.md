# Diem Framework Modules

This is the root document for the Diem framework module documentation. The Diem framework provides a set of Move
modules which define the resources and functions available for the Diem blockchain. Each module is individually
documented here, together with its implementation and
[formal specification](../../script_documentation/spec_documentation.md).

Move modules are not directly called by clients, but instead are used to implement *transaction scripts*.
For documentation of transaction scripts which constitute the client API, see
[../../script_documentation/script_documentation.md](../../script_documentation/script_documentation.md).

The Move modules in the Diem Framework can be bucketed in to a couple categories:

### Treasury and Compliance
* `AccountFreezing`
* `AccountLimits`
* `DesignatedDealer`
* `DualAttestation`

* `XUS`
* `XDX`
* `Diem`
* `RegisteredCurrencies`

### Authentication
* `Authenticator`
* `RecoveryAddress`
* `SharedEd25519PublicKey`
* `Signature`

### Accounts and Access Control
* `DiemAccount`
* `Roles`
* `VASP`

### System Management
* `ChainId`
* `DiemBlock`
* `DiemConfig`
* `DiemTimestamp`
* `DiemTransactionPublishingOption`
* `DiemVersion`
* `DiemVMConfig`
* `TransactionFee`
* `DiemSystem`
* `ValidatorConfig`
* `ValidatorOperatorConfig`
* `Genesis` (Note: not published on-chain)

### Module Utility Libraries
* `Errors`
* `CoreAddresses`
* `Event`
* `FixedPoint32`
* `Hash`
* `BCS`
* `Option`
* `SlidingNonce`
* `Vector`
* `Signer`

## Index

> {{move-index}}
