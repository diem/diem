
<a name="SCRIPT"></a>

# Script `create_recovery_address.move`

### Table of Contents

-  [Function `create_recovery_address`](#SCRIPT_create_recovery_address)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_create_recovery_address"></a>

## Function `create_recovery_address`


<a name="SCRIPT_@Summary"></a>

### Summary

Initializes the sending account as a recovery address that may be used by
the VASP that it belongs to. The sending account must be a VASP account.
Multiple recovery addresses can exist for a single VASP, but accounts in
each must be disjoint.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Publishes a <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource under <code>account</code>. It then
extracts the <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> for <code>account</code> and adds
it to the resource. After the successful execution of this transaction
other accounts may add their key rotation to this resource so that <code>account</code>
may be used as a recovery account for those accounts.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name      | Type      | Description                                           |
| ------    | ------    | -------------                                         |
| <code>account</code> | <code>&signer</code> | The signer of the sending account of the transaction. |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category              | Error Reason                                               | Description                                                                                   |
| ----------------            | --------------                                             | -------------                                                                                 |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>     | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>.          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ENOT_A_VASP">RecoveryAddress::ENOT_A_VASP</a></code>                             | <code>account</code> is not a VASP account.                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>  | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_EKEY_ROTATION_DEPENDENCY_CYCLE">RecoveryAddress::EKEY_ROTATION_DEPENDENCY_CYCLE</a></code>          | A key rotation recovery cycle would be created by adding <code>account</code>'s key rotation capability. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a></code> | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">RecoveryAddress::ERECOVERY_ADDRESS</a></code>                       | A <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource has already been published under <code>account</code>.     |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::add_recovery_rotation_capability</code>
* <code>Script::rotate_authentication_key_with_recovery_address</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_recovery_address">create_recovery_address</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_recovery_address">create_recovery_address</a>(account: &signer) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_publish">RecoveryAddress::publish</a>(account, <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account))
}
</code></pre>



</details>
