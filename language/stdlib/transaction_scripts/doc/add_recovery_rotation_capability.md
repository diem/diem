
<a name="SCRIPT"></a>

# Script `add_recovery_rotation_capability.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

Add the
<code>KeyRotationCapability</code> for
<code>to_recover_account</code> to the
<code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code>
resource under
<code>recovery_address</code>.
Aborts if
<code>to_recovery_account</code> and
<code>to_recovery_address belong <b>to</b> different VASPs, <b>if</b>
</code>recovery_address
<code> does not have a </code>RecoveryAddress
<code> <b>resource</b>, or <b>if</b>
</code>to_recover_account
<code> has already extracted its </code>KeyRotationCapability`.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(to_recover_account: &signer, recovery_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(to_recover_account: &signer, recovery_address: address) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_add_rotation_capability">RecoveryAddress::add_rotation_capability</a>(to_recover_account, recovery_address)
}
</code></pre>



</details>
