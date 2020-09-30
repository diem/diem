
<a name="create_recovery_address"></a>

# Script `create_recovery_address`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
-  [Parameters](#@Parameters_2)
-  [Common Abort Conditions](#@Common_Abort_Conditions_3)
-  [Related Scripts](#@Related_Scripts_4)


<a name="@Summary_0"></a>

## Summary

Initializes the sending account as a recovery address that may be used by
the VASP that it belongs to. The sending account must be a VASP account.
Multiple recovery addresses can exist for a single VASP, but accounts in
each must be disjoint.


<a name="@Technical_Description_1"></a>

## Technical Description

Publishes a <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource under <code>account</code>. It then
extracts the <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> for <code>account</code> and adds
it to the resource. After the successful execution of this transaction
other accounts may add their key rotation to this resource so that <code>account</code>
may be used as a recovery account for those accounts.


<a name="@Parameters_2"></a>

## Parameters

| Name      | Type      | Description                                           |
| ------    | ------    | -------------                                         |
| <code>account</code> | <code>&signer</code> | The signer of the sending account of the transaction. |


<a name="@Common_Abort_Conditions_3"></a>

## Common Abort Conditions

| Error Category              | Error Reason                                               | Description                                                                                   |
| ----------------            | --------------                                             | -------------                                                                                 |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>     | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ENOT_A_VASP">RecoveryAddress::ENOT_A_VASP</a></code>                             | <code>account</code> is not a VASP account.                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_EKEY_ROTATION_DEPENDENCY_CYCLE">RecoveryAddress::EKEY_ROTATION_DEPENDENCY_CYCLE</a></code>          | A key rotation recovery cycle would be created by adding <code>account</code>'s key rotation capability. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">RecoveryAddress::ERECOVERY_ADDRESS</a></code>                       | A <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource has already been published under <code>account</code>.     |


<a name="@Related_Scripts_4"></a>

## Related Scripts

* <code><a href="add_recovery_rotation_capability.md#add_recovery_rotation_capability">Script::add_recovery_rotation_capability</a></code>
* <code><a href="rotate_authentication_key_with_recovery_address.md#rotate_authentication_key_with_recovery_address">Script::rotate_authentication_key_with_recovery_address</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="create_recovery_address.md#create_recovery_address">create_recovery_address</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="create_recovery_address.md#create_recovery_address">create_recovery_address</a>(account: &signer) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_publish">RecoveryAddress::publish</a>(account, <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">LibraAccount::ExtractKeyRotationCapabilityAbortsIf</a>;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityEnsures">LibraAccount::ExtractKeyRotationCapabilityEnsures</a>;
<a name="create_recovery_address_addr$1"></a>
<b>let</b> addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<a name="create_recovery_address_rotation_cap$2"></a>
<b>let</b> rotation_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">LibraAccount::spec_get_key_rotation_cap</a>(addr);
<b>include</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_PublishAbortsIf">RecoveryAddress::PublishAbortsIf</a>{
    recovery_account: account,
    rotation_cap: rotation_cap
};
<b>ensures</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">RecoveryAddress::spec_is_recovery_address</a>(addr);
<b>ensures</b> len(<a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">RecoveryAddress::spec_get_rotation_caps</a>(addr)) == 1;
<b>ensures</b> <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">RecoveryAddress::spec_get_rotation_caps</a>(addr)[0] == <b>old</b>(rotation_cap);
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>



</details>
