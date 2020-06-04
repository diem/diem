
<a name="0x0_SharedEd25519PublicKey"></a>

# Module `0x0::SharedEd25519PublicKey`

### Table of Contents

-  [Struct `SharedEd25519PublicKey`](#0x0_SharedEd25519PublicKey_SharedEd25519PublicKey)
-  [Function `publish`](#0x0_SharedEd25519PublicKey_publish)
-  [Function `rotate_key_`](#0x0_SharedEd25519PublicKey_rotate_key_)
-  [Function `rotate_key`](#0x0_SharedEd25519PublicKey_rotate_key)
-  [Function `key`](#0x0_SharedEd25519PublicKey_key)
-  [Function `exists`](#0x0_SharedEd25519PublicKey_exists)



<a name="0x0_SharedEd25519PublicKey_SharedEd25519PublicKey"></a>

## Struct `SharedEd25519PublicKey`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_SharedEd25519PublicKey">SharedEd25519PublicKey</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>key: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>rotation_cap: <a href="LibraAccount.md#0x0_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_SharedEd25519PublicKey_publish"></a>

## Function `publish`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SharedEd25519PublicKey_publish">publish</a>(account: &signer, key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SharedEd25519PublicKey_publish">publish</a>(account: &signer, key: vector&lt;u8&gt;) {
    <b>let</b> t = <a href="#0x0_SharedEd25519PublicKey">SharedEd25519PublicKey</a> {
        key: x"",
        rotation_cap: <a href="LibraAccount.md#0x0_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(account)
    };
    <a href="#0x0_SharedEd25519PublicKey_rotate_key_">rotate_key_</a>(&<b>mut</b> t, key);
    move_to(account, t);
}
</code></pre>



</details>

<a name="0x0_SharedEd25519PublicKey_rotate_key_"></a>

## Function `rotate_key_`



<pre><code><b>fun</b> <a href="#0x0_SharedEd25519PublicKey_rotate_key_">rotate_key_</a>(shared_key: &<b>mut</b> <a href="#0x0_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a>, new_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_SharedEd25519PublicKey_rotate_key_">rotate_key_</a>(shared_key: &<b>mut</b> <a href="#0x0_SharedEd25519PublicKey">SharedEd25519PublicKey</a>, new_public_key: vector&lt;u8&gt;) {
    // Cryptographic check of <b>public</b> key validity
    Transaction::assert(
        <a href="Signature.md#0x0_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> new_public_key),
        9003, // TODO: proper error code
    );
    <a href="LibraAccount.md#0x0_LibraAccount_rotate_authentication_key_with_capability">LibraAccount::rotate_authentication_key_with_capability</a>(
        &shared_key.rotation_cap,
        <a href="Authenticator.md#0x0_Authenticator_ed25519_authentication_key">Authenticator::ed25519_authentication_key</a>(<b>copy</b> new_public_key)
    );
    shared_key.key = new_public_key;
}
</code></pre>



</details>

<a name="0x0_SharedEd25519PublicKey_rotate_key"></a>

## Function `rotate_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SharedEd25519PublicKey_rotate_key">rotate_key</a>(account: &signer, new_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SharedEd25519PublicKey_rotate_key">rotate_key</a>(account: &signer, new_public_key: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x0_SharedEd25519PublicKey">SharedEd25519PublicKey</a> {
    <a href="#0x0_SharedEd25519PublicKey_rotate_key_">rotate_key_</a>(borrow_global_mut&lt;<a href="#0x0_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account)), new_public_key);
}
</code></pre>



</details>

<a name="0x0_SharedEd25519PublicKey_key"></a>

## Function `key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SharedEd25519PublicKey_key">key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SharedEd25519PublicKey_key">key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x0_SharedEd25519PublicKey">SharedEd25519PublicKey</a> {
    *&borrow_global&lt;<a href="#0x0_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(addr).key
}
</code></pre>



</details>

<a name="0x0_SharedEd25519PublicKey_exists"></a>

## Function `exists`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SharedEd25519PublicKey_exists">exists</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_SharedEd25519PublicKey_exists">exists</a>(addr: address): bool {
    ::<a href="#0x0_SharedEd25519PublicKey_exists">exists</a>&lt;<a href="#0x0_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(addr)
}
</code></pre>



</details>
