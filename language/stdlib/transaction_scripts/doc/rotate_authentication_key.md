
<a name="SCRIPT"></a>

# Script `rotate_authentication_key.move`

### Table of Contents

-  [Function `rotate_authentication_key`](#SCRIPT_rotate_authentication_key)
-  [Specification](#SCRIPT_Specification)
    -  [Function `rotate_authentication_key`](#SCRIPT_Specification_rotate_authentication_key)



<a name="SCRIPT_rotate_authentication_key"></a>

## Function `rotate_authentication_key`

Rotate the sender's authentication key to
<code>new_key</code>.
<code>new_key</code> should be a 256 bit sha3 hash of an ed25519 public key.
* Aborts with
<code>LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</code> if the
<code>KeyRotationCapability</code> for
<code>account</code> has already been extracted.
* Aborts with
<code>0</code> if the key rotation capability held by the account doesn't match the sender's address.
* Aborts with
<code>LibraAccount::EMALFORMED_AUTHENTICATION_KEY</code> if the length of
<code>new_key</code> != 32.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;) {
  <b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account);
  <b>assert</b>(*<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">LibraAccount::key_rotation_capability_address</a>(&key_rotation_capability) == <a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account), 0);
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&key_rotation_capability, new_key);
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(key_rotation_capability);
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_rotate_authentication_key"></a>

### Function `rotate_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<a name="SCRIPT_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
</code></pre>


This rotates the authentication key of
<code>account</code> to
<code>new_key</code>


<pre><code><b>ensures</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_rotate_authentication_key">LibraAccount::spec_rotate_authentication_key</a>(account_addr, new_key);
</code></pre>


If the sending account doesn't exist this will abort


<pre><code><b>aborts_if</b> !exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>&gt;(account_addr);
</code></pre>



<code>account</code> must not have delegated its rotation capability


<pre><code><b>aborts_if</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_delegated_key_rotation_capability">LibraAccount::delegated_key_rotation_capability</a>(account_addr);
</code></pre>



<code>new_key</code>'s length must be
<code>32</code>.


<pre><code><b>aborts_if</b> len(new_key) != 32;
</code></pre>
