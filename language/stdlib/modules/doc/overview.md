
<a name="@Libra_Move_Framework_0"></a>

# Libra Move Framework


This is the root document for the Libra framework module documentation. The Libra framework provides a set of Move
modules which define the resources and functions available for the Libra blockchain. Each module is individually
documented here, together with it's implementation and [formal specification](../../../move-prover/doc/user/spec-lang.md).

Move modules are not directly called by clients, but instead are used to implement *transaction scripts*.
For documentation of transaction scripts which constitute the client API, see
[../../transaction_scripts/doc/overview.md](../../transaction_scripts/doc/overview.md).


<a name="@Index_1"></a>

## Index


-  [`0x1::AccountFreezing`](AccountFreezing.md#0x1_AccountFreezing)
-  [`0x1::AccountLimits`](AccountLimits.md#0x1_AccountLimits)
-  [`0x1::Authenticator`](Authenticator.md#0x1_Authenticator)
-  [`0x1::ChainId`](ChainId.md#0x1_ChainId)
-  [`0x1::Coin1`](Coin1.md#0x1_Coin1)
-  [`0x1::Compare`](Compare.md#0x1_Compare)
-  [`0x1::CoreAddresses`](CoreAddresses.md#0x1_CoreAddresses)
-  [`0x1::Debug`](Debug.md#0x1_Debug)
-  [`0x1::DesignatedDealer`](DesignatedDealer.md#0x1_DesignatedDealer)
-  [`0x1::DualAttestation`](DualAttestation.md#0x1_DualAttestation)
-  [`0x1::Errors`](Errors.md#0x1_Errors)
-  [`0x1::Event`](Event.md#0x1_Event)
-  [`0x1::FixedPoint32`](FixedPoint32.md#0x1_FixedPoint32)
-  [`0x1::Genesis`](Genesis.md#0x1_Genesis)
-  [`0x1::Hash`](Hash.md#0x1_Hash)
-  [`0x1::LBR`](LBR.md#0x1_LBR)
-  [`0x1::LCS`](LCS.md#0x1_LCS)
-  [`0x1::Libra`](Libra.md#0x1_Libra)
-  [`0x1::LibraAccount`](LibraAccount.md#0x1_LibraAccount)
-  [`0x1::LibraBlock`](LibraBlock.md#0x1_LibraBlock)
-  [`0x1::LibraConfig`](LibraConfig.md#0x1_LibraConfig)
-  [`0x1::LibraSystem`](LibraSystem.md#0x1_LibraSystem)
-  [`0x1::LibraTimestamp`](LibraTimestamp.md#0x1_LibraTimestamp)
-  [`0x1::LibraTransactionPublishingOption`](LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption)
-  [`0x1::LibraVMConfig`](LibraVMConfig.md#0x1_LibraVMConfig)
-  [`0x1::LibraVersion`](LibraVersion.md#0x1_LibraVersion)
-  [`0x1::Offer`](Offer.md#0x1_Offer)
-  [`0x1::Option`](Option.md#0x1_Option)
-  [`0x1::RecoveryAddress`](RecoveryAddress.md#0x1_RecoveryAddress)
-  [`0x1::RegisteredCurrencies`](RegisteredCurrencies.md#0x1_RegisteredCurrencies)
-  [`0x1::Roles`](Roles.md#0x1_Roles)
-  [`0x1::SharedEd25519PublicKey`](SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey)
-  [`0x1::Signature`](Signature.md#0x1_Signature)
-  [`0x1::Signer`](Signer.md#0x1_Signer)
-  [`0x1::SlidingNonce`](SlidingNonce.md#0x1_SlidingNonce)
-  [`0x1::TransactionFee`](TransactionFee.md#0x1_TransactionFee)
-  [`0x1::VASP`](VASP.md#0x1_VASP)
-  [`0x1::ValidatorConfig`](ValidatorConfig.md#0x1_ValidatorConfig)
-  [`0x1::ValidatorOperatorConfig`](ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig)
-  [`0x1::Vector`](Vector.md#0x1_Vector)


[//]: # ("File containing references which can be used from documentation")
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
