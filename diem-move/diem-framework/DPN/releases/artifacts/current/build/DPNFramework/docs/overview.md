
<a name="@Diem_Framework_Modules_0"></a>

# Diem Framework Modules


This is the root document for the Diem framework module documentation. The Diem framework provides a set of Move
modules which define the resources and functions available for the Diem blockchain. Each module is individually
documented here, together with its implementation and
[formal specification](../../script_documentation/spec_documentation.md).

Move modules are not directly called by clients, but instead are used to implement *transaction scripts*.
For documentation of transaction scripts which constitute the client API, see
[../../script_documentation/script_documentation.md](../../script_documentation/script_documentation.md).

The Move modules in the Diem Framework can be bucketed in to a couple categories:


<a name="@Treasury_and_Compliance_1"></a>

### Treasury and Compliance

* <code><a href="AccountFreezing.md#0x1_AccountFreezing">AccountFreezing</a></code>
* <code><a href="AccountLimits.md#0x1_AccountLimits">AccountLimits</a></code>
* <code><a href="DesignatedDealer.md#0x1_DesignatedDealer">DesignatedDealer</a></code>
* <code><a href="DualAttestation.md#0x1_DualAttestation">DualAttestation</a></code>

* <code><a href="XUS.md#0x1_XUS">XUS</a></code>
* <code><a href="XDX.md#0x1_XDX">XDX</a></code>
* <code><a href="Diem.md#0x1_Diem">Diem</a></code>
* <code><a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a></code>


<a name="@Authentication_2"></a>

### Authentication

* <code><a href="Authenticator.md#0x1_Authenticator">Authenticator</a></code>
* <code><a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code>
* <code><a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code>
* <code><a href="Signature.md#0x1_Signature">Signature</a></code>


<a name="@Accounts_and_Access_Control_3"></a>

### Accounts and Access Control

* <code><a href="DiemAccount.md#0x1_DiemAccount">DiemAccount</a></code>
* <code><a href="Roles.md#0x1_Roles">Roles</a></code>
* <code><a href="VASP.md#0x1_VASP">VASP</a></code>


<a name="@System_Management_4"></a>

### System Management

* <code><a href="ChainId.md#0x1_ChainId">ChainId</a></code>
* <code><a href="DiemBlock.md#0x1_DiemBlock">DiemBlock</a></code>
* <code><a href="DiemConfig.md#0x1_DiemConfig">DiemConfig</a></code>
* <code><a href="DiemTimestamp.md#0x1_DiemTimestamp">DiemTimestamp</a></code>
* <code><a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a></code>
* <code><a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a></code>
* <code><a href="DiemVMConfig.md#0x1_DiemVMConfig">DiemVMConfig</a></code>
* <code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code>
* <code><a href="DiemSystem.md#0x1_DiemSystem">DiemSystem</a></code>
* <code><a href="ValidatorConfig.md#0x1_ValidatorConfig">ValidatorConfig</a></code>
* <code><a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig">ValidatorOperatorConfig</a></code>
* <code><a href="Genesis.md#0x1_Genesis">Genesis</a></code> (Note: not published on-chain)


<a name="@Module_Utility_Libraries_5"></a>

### Module Utility Libraries

* <code><a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">Errors</a></code>
* <code><a href="CoreAddresses.md#0x1_CoreAddresses">CoreAddresses</a></code>
* <code><a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event">Event</a></code>
* <code><a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code>
* <code><a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Hash.md#0x1_Hash">Hash</a></code>
* <code><a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/BCS.md#0x1_BCS">BCS</a></code>
* <code><a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option">Option</a></code>
* <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>
* <code><a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">Vector</a></code>
* <code><a href="../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">Signer</a></code>


<a name="@Index_6"></a>

## Index


-  [`0x1::AccountAdministrationScripts`](script_documentation.md#0x1_AccountAdministrationScripts)
-  [`0x1::AccountCreationScripts`](script_documentation.md#0x1_AccountCreationScripts)
-  [`0x1::AccountFreezing`](AccountFreezing.md#0x1_AccountFreezing)
-  [`0x1::AccountLimits`](AccountLimits.md#0x1_AccountLimits)
-  [`0x1::Authenticator`](Authenticator.md#0x1_Authenticator)
-  [`0x1::BCS`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/BCS.md#0x1_BCS)
-  [`0x1::BitVector`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector)
-  [`0x1::CRSN`](CRSN.md#0x1_CRSN)
-  [`0x1::ChainId`](ChainId.md#0x1_ChainId)
-  [`0x1::CoreAddresses`](CoreAddresses.md#0x1_CoreAddresses)
-  [`0x1::DesignatedDealer`](DesignatedDealer.md#0x1_DesignatedDealer)
-  [`0x1::Diem`](Diem.md#0x1_Diem)
-  [`0x1::DiemAccount`](DiemAccount.md#0x1_DiemAccount)
-  [`0x1::DiemBlock`](DiemBlock.md#0x1_DiemBlock)
-  [`0x1::DiemConfig`](DiemConfig.md#0x1_DiemConfig)
-  [`0x1::DiemConsensusConfig`](DiemConsensusConfig.md#0x1_DiemConsensusConfig)
-  [`0x1::DiemId`](DiemId.md#0x1_DiemId)
-  [`0x1::DiemSystem`](DiemSystem.md#0x1_DiemSystem)
-  [`0x1::DiemTimestamp`](DiemTimestamp.md#0x1_DiemTimestamp)
-  [`0x1::DiemTransactionPublishingOption`](DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption)
-  [`0x1::DiemVMConfig`](DiemVMConfig.md#0x1_DiemVMConfig)
-  [`0x1::DiemVersion`](DiemVersion.md#0x1_DiemVersion)
-  [`0x1::DualAttestation`](DualAttestation.md#0x1_DualAttestation)
-  [`0x1::Errors`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors)
-  [`0x1::Event`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event)
-  [`0x1::FixedPoint32`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32)
-  [`0x1::GUID`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/GUID.md#0x1_GUID)
-  [`0x1::Genesis`](Genesis.md#0x1_Genesis)
-  [`0x1::Hash`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Hash.md#0x1_Hash)
-  [`0x1::NetworkIdentity`](NetworkIdentity.md#0x1_NetworkIdentity)
-  [`0x1::Option`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Option.md#0x1_Option)
-  [`0x1::ParallelExecutionConfig`](ParallelExecutionConfig.md#0x1_ParallelExecutionConfig)
-  [`0x1::PaymentScripts`](script_documentation.md#0x1_PaymentScripts)
-  [`0x1::RecoveryAddress`](RecoveryAddress.md#0x1_RecoveryAddress)
-  [`0x1::RegisteredCurrencies`](RegisteredCurrencies.md#0x1_RegisteredCurrencies)
-  [`0x1::Roles`](Roles.md#0x1_Roles)
-  [`0x1::SharedEd25519PublicKey`](SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey)
-  [`0x1::Signature`](Signature.md#0x1_Signature)
-  [`0x1::Signer`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer)
-  [`0x1::SlidingNonce`](SlidingNonce.md#0x1_SlidingNonce)
-  [`0x1::SystemAdministrationScripts`](script_documentation.md#0x1_SystemAdministrationScripts)
-  [`0x1::TransactionFee`](TransactionFee.md#0x1_TransactionFee)
-  [`0x1::TreasuryComplianceScripts`](script_documentation.md#0x1_TreasuryComplianceScripts)
-  [`0x1::VASP`](VASP.md#0x1_VASP)
-  [`0x1::VASPDomain`](VASPDomain.md#0x1_VASPDomain)
-  [`0x1::ValidatorAdministrationScripts`](script_documentation.md#0x1_ValidatorAdministrationScripts)
-  [`0x1::ValidatorConfig`](ValidatorConfig.md#0x1_ValidatorConfig)
-  [`0x1::ValidatorOperatorConfig`](ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig)
-  [`0x1::Vector`](../../../../../../../DPN/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector)
-  [`0x1::XDX`](XDX.md#0x1_XDX)
-  [`0x1::XUS`](XUS.md#0x1_XUS)


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
