
<a name="SCRIPT"></a>

# Script `rotate_authentication_key_with_recovery_address.move`

### Table of Contents

-  [Function `rotate_authentication_key_with_recovery_address`](#SCRIPT_rotate_authentication_key_with_recovery_address)



<a name="SCRIPT_rotate_authentication_key_with_recovery_address"></a>

## Function `rotate_authentication_key_with_recovery_address`

Rotate the authentication key of
<code>to_recover</code> to
<code>new_key</code>. Can be invoked by either
<code>recovery_address</code> or
<code>to_recover</code>. Aborts if
<code>recovery_address</code> does not have the
<code>KeyRotationCapability</code> for
<code>to_recover</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_rotate_authentication_key">RecoveryAddress::rotate_authentication_key</a>(account, recovery_address, to_recover, new_key)
}
</code></pre>



</details>
