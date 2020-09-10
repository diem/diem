
<a name="SCRIPT"></a>

# Script `create_recovery_address.move`

### Table of Contents

-  [Function `create_recovery_address`](#SCRIPT_create_recovery_address)
        -  [Aborts](#SCRIPT_@Aborts)



<a name="SCRIPT_create_recovery_address"></a>

## Function `create_recovery_address`

Extract the
<code>KeyRotationCapability</code> for
<code>recovery_account</code> and publish it in a
<code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource under
<code>account</code>.

<a name="SCRIPT_@Aborts"></a>

#### Aborts

* Aborts with
<code>LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</code> if
<code>account</code> has already delegated its
<code>KeyRotationCapability</code>.
* Aborts with
<code>RecoveryAddress::ENOT_A_VASP</code> if
<code>account</code> is not a ParentVASP or ChildVASP


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_recovery_address">create_recovery_address</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_recovery_address">create_recovery_address</a>(account: &signer) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_publish">RecoveryAddress::publish</a>(account, <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account))
}
</code></pre>



</details>
