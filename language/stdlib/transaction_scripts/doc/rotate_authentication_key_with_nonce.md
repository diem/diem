
<a name="SCRIPT"></a>

# Script `rotate_authentication_key_with_nonce.move`

### Table of Contents

-  [Function `rotate_authentication_key_with_nonce`](#SCRIPT_rotate_authentication_key_with_nonce)



<a name="SCRIPT_rotate_authentication_key_with_nonce"></a>

## Function `rotate_authentication_key_with_nonce`

Rotate
<code>account</code>'s authentication key to
<code>new_key</code>.
<code>new_key</code> should be a 256 bit sha3 hash of an ed25519 public key. This script also takes
<code>sliding_nonce</code>, as a unique nonce for this operation. See sliding_nonce.move for details.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_nonce">rotate_authentication_key_with_nonce</a>(account: &signer, sliding_nonce: u64, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_nonce">rotate_authentication_key_with_nonce</a>(account: &signer, sliding_nonce: u64, new_key: vector&lt;u8&gt;) {
  <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
  <b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account);
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&key_rotation_capability, new_key);
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(key_rotation_capability);
}
</code></pre>



</details>
