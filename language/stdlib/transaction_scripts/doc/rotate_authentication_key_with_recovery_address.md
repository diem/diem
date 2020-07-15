
<a name="SCRIPT"></a>

# Script `rotate_authentication_key_with_recovery_address.move`

### Table of Contents

-  [Function `rotate_authentication_key_with_recovery_address`](#SCRIPT_rotate_authentication_key_with_recovery_address)
        -  [Aborts](#SCRIPT_@Aborts)



<a name="SCRIPT_rotate_authentication_key_with_recovery_address"></a>

## Function `rotate_authentication_key_with_recovery_address`

Rotate the authentication key of
<code>account</code> to
<code>new_key</code> using the
<code>KeyRotationCapability</code>
stored under
<code>recovery_address</code>.


<a name="SCRIPT_@Aborts"></a>

#### Aborts

* Aborts with
<code>RecoveryAddress::ENOT_A_RECOVERY_ADDRESS</code> if
<code>recovery_address</code> does not have a
<code><a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource
* Aborts with
<code>RecoveryAddress::ECANNOT_ROTATE_KEY</code> if
<code>account</code> is not
<code>recovery_address</code> or
<code>to_recover</code>.
* Aborts with
<code>LibraAccount::EMALFORMED_AUTHENTICATION_KEY</code> if
<code>new_key</code> is not 32 bytes.
* Aborts with
<code>RecoveryAddress::ECANNOT_ROTATE_KEY</code> if
<code>account</code> has not delegated its
<code>KeyRotationCapability</code> to
<code>recovery_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_recovery_address">rotate_authentication_key_with_recovery_address</a>(
    account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;
) {
    <a href="../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress_rotate_authentication_key">RecoveryAddress::rotate_authentication_key</a>(account, recovery_address, to_recover, new_key)
}
</code></pre>



</details>
