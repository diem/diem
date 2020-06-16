
<a name="SCRIPT"></a>

# Script `create_recovery_address.move`

### Table of Contents

-  [Function `create_recovery_address`](#SCRIPT_create_recovery_address)



<a name="SCRIPT_create_recovery_address"></a>

## Function `create_recovery_address`

Extract the
<code>KeyRotationCapability</code> for
<code>recovery_account</code> and publish it in a
<code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource under
<code>recovery_account</code>.
Aborts if
<code>recovery_account</code> has delegated its
<code>KeyRotationCapability</code>, already has a
<code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource, or is not a VASP.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_create_recovery_address">create_recovery_address</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_create_recovery_address">create_recovery_address</a>(account: &signer) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_publish">RecoveryAddress::publish</a>(account)
}
</code></pre>



</details>
