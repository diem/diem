
<a name="SCRIPT"></a>

# Script `add_recovery_rotation_capability.move`

### Table of Contents

-  [Function `add_recovery_rotation_capability`](#SCRIPT_add_recovery_rotation_capability)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_add_recovery_rotation_capability"></a>

## Function `add_recovery_rotation_capability`


<a name="SCRIPT_@Summary"></a>

### Summary

Stores the sending accounts ability to rotate its authentication key with a designated recovery
account. Both the sending and recovery accounts need to belong to the same VASP and
both be VASP accounts. After this transaction both the sending account and the
specified recovery account can rotate the sender account's authentication key.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Adds the <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> for the sending account
(<code>to_recover_account</code>) to the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code> resource under
<code>recovery_address</code>. After this transaction has been executed successfully the account at
<code>recovery_address</code> and the <code>to_recover_account</code> may rotate the authentication key of
<code>to_recover_account</code> (the sender of this transaction).

The sending account of this transaction (<code>to_recover_account</code>) must not have previously given away its unique key
rotation capability, and must be a VASP account. The account at <code>recovery_address</code>
must also be a VASP account belonging to the same VASP as the <code>to_recover_account</code>.
Additionally the account at <code>recovery_address</code> must have already initialized itself as
a recovery account address using the <code>Script::create_recovery_address</code> transaction script.

The sending account's (<code>to_recover_account</code>) key rotation capability is
removed in this transaction and stored in the <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_RecoveryAddress">RecoveryAddress::RecoveryAddress</a></code>
resource stored under the account at <code>recovery_address</code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name                 | Type      | Description                                                                                                |
| ------               | ------    | -------------                                                                                              |
| <code>to_recover_account</code> | <code>&signer</code> | The signer reference of the sending account of this transaction.                                           |
| <code>recovery_address</code>   | <code>address</code> | The account address where the <code>to_recover_account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code> will be stored. |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                               | Description                                                                                     |
| ----------------           | --------------                                             | -------------                                                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>to_recover_account</code> has already delegated/extracted its <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">RecoveryAddress::ERECOVERY_ADDRESS</a></code>                       | <code>recovery_address</code> does not have a <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource published under it.               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_EINVALID_KEY_ROTATION_DELEGATION">RecoveryAddress::EINVALID_KEY_ROTATION_DELEGATION</a></code>        | <code>to_recover_account</code> and <code>recovery_address</code> do not belong to the same VASP.                     |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::create_recovery_address</code>
* <code>Script::rotate_authentication_key_with_recovery_address</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_recovery_rotation_capability">add_recovery_rotation_capability</a>(to_recover_account: &signer, recovery_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_recovery_rotation_capability">add_recovery_rotation_capability</a>(to_recover_account: &signer, recovery_address: address) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_add_rotation_capability">RecoveryAddress::add_rotation_capability</a>(
        <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(to_recover_account), recovery_address
    )
}
</code></pre>



</details>
