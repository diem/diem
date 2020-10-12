# Libra Framework Modules

This is the root document for the Libra framework module documentation. The Libra framework provides a set of Move
modules which define the resources and functions available for the Libra blockchain. Each module is individually
documented here, together with it's implementation and [formal specification](../../../move-prover/doc/user/spec-lang.md).

Move modules are not directly called by clients, but instead are used to implement *transaction scripts*.
For documentation of transaction scripts which constitute the client API, see
[../../transaction_scripts/doc/transaction_script_documentation.md](../../transaction_scripts/doc/transaction_script_documentation.md).

Move modules come together with formal specifications. See [this document](../../transaction_scripts/doc/spec_documentation.md)
for a discussion of specifications and pointers to further documentation.

The Move modules in the Libra Framework can be bucketed in to a couple categories:

### Treasury and Compliance
* `AccountFreezing`
* `AccountLimits`
* `DesignatedDealer`
* `DualAttestation`

* `Coin1` (Note: name will be updated once final name has been determined)
* `LBR` (Note: will be updated once the LBR makeup has been determined)
* `Libra`
* `RegisteredCurrencies`

### Authentication
* `Authenticator`
* `RecoveryAddress`
* `SharedEd25519PublicKey`
* `Signature`

### Accounts and Access Control
* `LibraAccount`
* `Roles`
* `VASP`

### System Management
* `ChainId`
* `LibraBlock`
* `LibraConfig`
* `LibraTimestamp`
* `LibraTransactionPublishingOption`
* `LibraVersion`
* `LibraVMConfig`
* `TransactionFee`
* `LibraSystem`
* `ValidatorConfig`
* `ValidatorOperatorConfig`
* `Genesis` (Note: not published)

### Module Utility Libraries
* `Errors`
* `CoreAddresses`
* `Event`
* `FixedPoint32`
* `Hash`
* `LCS`
* `Offer`
* `Option`
* `SlidingNonce`
* `Vector`
* `Signer`

## Index

> {{move-index}}
