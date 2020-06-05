
<a name="SCRIPT"></a>

# Script `rotate_authentication_key_with_recovery_address.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

Extract the
<code>KeyRotationCapability</code> for
<code>recovery_account</code> and publish it in a
<code><a href="../../modules/doc/RecoveryAddress.md#0x0_RecoveryAddress">RecoveryAddress</a></code> resource under
<code>recovery_account</code>.
Aborts if
<code>recovery_account</code> has delegated its
<code>KeyRotationCapability</code>, already has a
<code><a href="../../modules/doc/RecoveryAddress.md#0x0_RecoveryAddress">RecoveryAddress</a></code> resource, or is not a VASP.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/RecoveryAddress.md#0x0_RecoveryAddress_rotate_authentication_key">RecoveryAddress::rotate_authentication_key</a>(account, recovery_address, to_recover, new_key)
}
</code></pre>



</details>
