
<a name="SCRIPT"></a>

# Script `add_recovery_rotation_capability.move`

### Table of Contents

-  [Function `add_recovery_rotation_capability`](#SCRIPT_add_recovery_rotation_capability)
        -  [Aborts](#SCRIPT_@Aborts)



<a name="SCRIPT_add_recovery_rotation_capability"></a>

## Function `add_recovery_rotation_capability`

Add the
<code>KeyRotationCapability</code> for
<code>to_recover_account</code> to the
<code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource under
<code>recovery_address</code>.


<a name="SCRIPT_@Aborts"></a>

#### Aborts

* Aborts with
<code>LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</code> if
<code>account</code> has already delegated its
<code>KeyRotationCapability</code>.
* Aborts with
<code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>:ENOT_A_RECOVERY_ADDRESS</code> if
<code>recovery_address</code> does not have a
<code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource.
* Aborts with
<code>RecoveryAddress::EINVALID_KEY_ROTATION_DELEGATION</code> if
<code>to_recover_account</code> and
<code>recovery_address</code> do not belong to the same VASP.


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
