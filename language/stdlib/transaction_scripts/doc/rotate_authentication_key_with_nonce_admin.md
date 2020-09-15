
<a name="SCRIPT"></a>

# Script `rotate_authentication_key_with_nonce_admin.move`

### Table of Contents

-  [Function `rotate_authentication_key_with_nonce_admin`](#SCRIPT_rotate_authentication_key_with_nonce_admin)
-  [Specification](#SCRIPT_Specification)
    -  [Function `rotate_authentication_key_with_nonce_admin`](#SCRIPT_Specification_rotate_authentication_key_with_nonce_admin)



<a name="SCRIPT_rotate_authentication_key_with_nonce_admin"></a>

## Function `rotate_authentication_key_with_nonce_admin`

Rotate
<code>account</code>'s authentication key to
<code>new_key</code>.
<code>new_key</code> should be a 256 bit sha3 hash of an ed25519 public key. This script also takes
<code>sliding_nonce</code>, as a unique nonce for this operation. See sliding_nonce.move for details.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_nonce_admin">rotate_authentication_key_with_nonce_admin</a>(lr_account: &signer, account: &signer, sliding_nonce: u64, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_nonce_admin">rotate_authentication_key_with_nonce_admin</a>(lr_account: &signer, account: &signer, sliding_nonce: u64, new_key: vector&lt;u8&gt;) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&key_rotation_capability, new_key);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(key_rotation_capability);
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_rotate_authentication_key_with_nonce_admin"></a>

### Function `rotate_authentication_key_with_nonce_admin`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_rotate_authentication_key_with_nonce_admin">rotate_authentication_key_with_nonce_admin</a>(lr_account: &signer, account: &signer, sliding_nonce: u64, new_key: vector&lt;u8&gt;)
</code></pre>




<a name="SCRIPT_account_addr$1"></a>


<pre><code><b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{ account: lr_account, seq_nonce: sliding_nonce };
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">LibraAccount::ExtractKeyRotationCapabilityAbortsIf</a>;
<a name="SCRIPT_key_rotation_capability$2"></a>
<b>let</b> key_rotation_capability = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">LibraAccount::spec_get_key_rotation_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyAbortsIf">LibraAccount::RotateAuthenticationKeyAbortsIf</a>{cap: key_rotation_capability, new_authentication_key: new_key};
</code></pre>


This rotates the authentication key of
<code>account</code> to
<code>new_key</code>


<pre><code><b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyEnsures">LibraAccount::RotateAuthenticationKeyEnsures</a>{addr: account_addr, new_authentication_key: new_key};
</code></pre>
