
<a name="0x0_Authenticator"></a>

# Module `0x0::Authenticator`

### Table of Contents

-  [Struct `MultiEd25519PublicKey`](#0x0_Authenticator_MultiEd25519PublicKey)
-  [Function `create_multi_ed25519`](#0x0_Authenticator_create_multi_ed25519)
-  [Function `ed25519_authentication_key`](#0x0_Authenticator_ed25519_authentication_key)
-  [Function `multi_ed25519_authentication_key`](#0x0_Authenticator_multi_ed25519_authentication_key)
-  [Function `public_keys`](#0x0_Authenticator_public_keys)
-  [Function `threshold`](#0x0_Authenticator_threshold)



<a name="0x0_Authenticator_MultiEd25519PublicKey"></a>

## Struct `MultiEd25519PublicKey`



<pre><code><b>struct</b> <a href="#0x0_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>public_keys: vector&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>threshold: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Authenticator_create_multi_ed25519"></a>

## Function `create_multi_ed25519`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_create_multi_ed25519">create_multi_ed25519</a>(public_keys: vector&lt;vector&lt;u8&gt;&gt;, threshold: u8): <a href="#0x0_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_create_multi_ed25519">create_multi_ed25519</a>(
    public_keys: vector&lt;vector&lt;u8&gt;&gt;,
    threshold: u8
): <a href="#0x0_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a> {
    // check theshold requirements
    <b>let</b> len = <a href="vector.md#0x0_Vector_length">Vector::length</a>(&public_keys);
    Transaction::assert(threshold != 0, 7001);
    Transaction::assert((threshold <b>as</b> u64) &lt;= len, 7002);
    // TODO: add constant MULTI_ED25519_MAX_KEYS
    // the multied25519 signature scheme allows at most 32 keys
    Transaction::assert(len &lt;= 32, 7003);

    <a href="#0x0_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a> { public_keys, threshold }
}
</code></pre>



</details>

<a name="0x0_Authenticator_ed25519_authentication_key"></a>

## Function `ed25519_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_ed25519_authentication_key">ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_ed25519_authentication_key">ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt; {
    // TODO: add constant ED25519_SCHEME_ID = 0u8
    <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(&<b>mut</b> public_key, 0u8);
    <a href="hash.md#0x0_Hash_sha3_256">Hash::sha3_256</a>(public_key)
}
</code></pre>



</details>

<a name="0x0_Authenticator_multi_ed25519_authentication_key"></a>

## Function `multi_ed25519_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_multi_ed25519_authentication_key">multi_ed25519_authentication_key</a>(k: &<a href="#0x0_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_multi_ed25519_authentication_key">multi_ed25519_authentication_key</a>(k: &<a href="#0x0_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): vector&lt;u8&gt; {
    <b>let</b> public_keys = &k.public_keys;
    <b>let</b> len = <a href="vector.md#0x0_Vector_length">Vector::length</a>(public_keys);
    <b>let</b> authentication_key_preimage = <a href="vector.md#0x0_Vector_empty">Vector::empty</a>();
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len) {
        <b>let</b> public_key = *<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(public_keys, i);
        <a href="vector.md#0x0_Vector_append">Vector::append</a>(
            &<b>mut</b> authentication_key_preimage,
            public_key
        );
        i = i + 1;
    };
    <a href="vector.md#0x0_Vector_append">Vector::append</a>(&<b>mut</b> authentication_key_preimage, <a href="lcs.md#0x0_LCS_to_bytes">LCS::to_bytes</a>(&k.threshold));
    // TODO: add constant MULTI_ED25519_SCHEME_ID = 1u8
    <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(&<b>mut</b> authentication_key_preimage, 1u8);
    <a href="hash.md#0x0_Hash_sha3_256">Hash::sha3_256</a>(authentication_key_preimage)
}
</code></pre>



</details>

<a name="0x0_Authenticator_public_keys"></a>

## Function `public_keys`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_public_keys">public_keys</a>(k: &<a href="#0x0_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): &vector&lt;vector&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_public_keys">public_keys</a>(k: &<a href="#0x0_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): &vector&lt;vector&lt;u8&gt;&gt; {
    &k.public_keys
}
</code></pre>



</details>

<a name="0x0_Authenticator_threshold"></a>

## Function `threshold`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_threshold">threshold</a>(k: &<a href="#0x0_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Authenticator_threshold">threshold</a>(k: &<a href="#0x0_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): u8 {
    *&k.threshold
}
</code></pre>



</details>
