
<a name="0x1_Authenticator"></a>

# Module `0x1::Authenticator`

### Table of Contents

-  [Struct `MultiEd25519PublicKey`](#0x1_Authenticator_MultiEd25519PublicKey)
-  [Const `SINGLE_ED25519_SCHEME_ID`](#0x1_Authenticator_SINGLE_ED25519_SCHEME_ID)
-  [Const `MULTI_ED25519_SCHEME_ID`](#0x1_Authenticator_MULTI_ED25519_SCHEME_ID)
-  [Const `MAX_MULTI_ED25519_KEYS`](#0x1_Authenticator_MAX_MULTI_ED25519_KEYS)
-  [Const `EZERO_THRESHOLD`](#0x1_Authenticator_EZERO_THRESHOLD)
-  [Const `ENOT_ENOUGH_KEYS_FOR_THRESHOLD`](#0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD)
-  [Const `ENUM_KEYS_ABOVE_MAX_THRESHOLD`](#0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD)
-  [Function `create_multi_ed25519`](#0x1_Authenticator_create_multi_ed25519)
-  [Function `ed25519_authentication_key`](#0x1_Authenticator_ed25519_authentication_key)
-  [Function `multi_ed25519_authentication_key`](#0x1_Authenticator_multi_ed25519_authentication_key)
-  [Function `public_keys`](#0x1_Authenticator_public_keys)
-  [Function `threshold`](#0x1_Authenticator_threshold)

Move representation of the authenticator types used in Libra:
- Ed25519 (single-sig)
- MultiEd25519 (K-of-N multisig)


<a name="0x1_Authenticator_MultiEd25519PublicKey"></a>

## Struct `MultiEd25519PublicKey`

A multi-ed25519 public key


<pre><code><b>struct</b> <a href="#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>public_keys: vector&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>
 vector of ed25519 public keys
</dd>
<dt>

<code>threshold: u8</code>
</dt>
<dd>
 approval threshold
</dd>
</dl>


</details>

<a name="0x1_Authenticator_SINGLE_ED25519_SCHEME_ID"></a>

## Const `SINGLE_ED25519_SCHEME_ID`

Scheme byte ID for ed25519


<pre><code><b>const</b> SINGLE_ED25519_SCHEME_ID: u8 = 0;
</code></pre>



<a name="0x1_Authenticator_MULTI_ED25519_SCHEME_ID"></a>

## Const `MULTI_ED25519_SCHEME_ID`

Scheme byte ID for multi-ed25519


<pre><code><b>const</b> MULTI_ED25519_SCHEME_ID: u8 = 1;
</code></pre>



<a name="0x1_Authenticator_MAX_MULTI_ED25519_KEYS"></a>

## Const `MAX_MULTI_ED25519_KEYS`

Maximum number of keys allowed in a MultiEd25519 public/private key


<pre><code><b>const</b> MAX_MULTI_ED25519_KEYS: u64 = 32;
</code></pre>



<a name="0x1_Authenticator_EZERO_THRESHOLD"></a>

## Const `EZERO_THRESHOLD`

Threshold provided was 0 which can't be used to create a
<code>MultiEd25519</code> key


<pre><code><b>const</b> EZERO_THRESHOLD: u64 = 0;
</code></pre>



<a name="0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD"></a>

## Const `ENOT_ENOUGH_KEYS_FOR_THRESHOLD`

Not enough keys were provided for the specified threshold when creating an
<code>MultiEd25519</code> key


<pre><code><b>const</b> ENOT_ENOUGH_KEYS_FOR_THRESHOLD: u64 = 1;
</code></pre>



<a name="0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD"></a>

## Const `ENUM_KEYS_ABOVE_MAX_THRESHOLD`

Too many keys were provided for the specified threshold when creating an
<code>MultiEd25519</code> key


<pre><code><b>const</b> ENUM_KEYS_ABOVE_MAX_THRESHOLD: u64 = 2;
</code></pre>



<a name="0x1_Authenticator_create_multi_ed25519"></a>

## Function `create_multi_ed25519`

Create a a multisig policy from a vector of ed25519 public keys and a threshold.
Note: this does *not* check uniqueness of keys. Repeated keys are convenient to
encode weighted multisig policies. For example Alice AND 1 of Bob or Carol is
public_key: {alice_key, alice_key, bob_key, carol_key}, threshold: 3
Aborts if threshold is zero or bigger than the length of
<code>public_keys</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_create_multi_ed25519">create_multi_ed25519</a>(public_keys: vector&lt;vector&lt;u8&gt;&gt;, threshold: u8): <a href="#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_create_multi_ed25519">create_multi_ed25519</a>(
    public_keys: vector&lt;vector&lt;u8&gt;&gt;,
    threshold: u8
): <a href="#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a> {
    // check theshold requirements
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&public_keys);
    <b>assert</b>(threshold != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(EZERO_THRESHOLD));
    <b>assert</b>(
        (threshold <b>as</b> u64) &lt;= len,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(ENOT_ENOUGH_KEYS_FOR_THRESHOLD)
    );
    // the multied25519 signature scheme allows at most 32 keys
    <b>assert</b>(
        len &lt;= MAX_MULTI_ED25519_KEYS,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(ENUM_KEYS_ABOVE_MAX_THRESHOLD)
    );

    <a href="#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a> { public_keys, threshold }
}
</code></pre>



</details>

<a name="0x1_Authenticator_ed25519_authentication_key"></a>

## Function `ed25519_authentication_key`

Compute an authentication key for the ed25519 public key
<code>public_key</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_ed25519_authentication_key">ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_ed25519_authentication_key">ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt; {
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> public_key, SINGLE_ED25519_SCHEME_ID);
    <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(public_key)
}
</code></pre>



</details>

<a name="0x1_Authenticator_multi_ed25519_authentication_key"></a>

## Function `multi_ed25519_authentication_key`

Compute a multied25519 account authentication key for the policy
<code>k</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_multi_ed25519_authentication_key">multi_ed25519_authentication_key</a>(k: &<a href="#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_multi_ed25519_authentication_key">multi_ed25519_authentication_key</a>(k: &<a href="#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): vector&lt;u8&gt; {
    <b>let</b> public_keys = &k.public_keys;
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(public_keys);
    <b>let</b> authentication_key_preimage = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len) {
        <b>let</b> public_key = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(public_keys, i);
        <a href="Vector.md#0x1_Vector_append">Vector::append</a>(
            &<b>mut</b> authentication_key_preimage,
            public_key
        );
        i = i + 1;
    };
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> authentication_key_preimage, <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&k.threshold));
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> authentication_key_preimage, MULTI_ED25519_SCHEME_ID);
    <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(authentication_key_preimage)
}
</code></pre>



</details>

<a name="0x1_Authenticator_public_keys"></a>

## Function `public_keys`

Return the public keys involved in the multisig policy
<code>k</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_public_keys">public_keys</a>(k: &<a href="#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): &vector&lt;vector&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_public_keys">public_keys</a>(k: &<a href="#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): &vector&lt;vector&lt;u8&gt;&gt; {
    &k.public_keys
}
</code></pre>



</details>

<a name="0x1_Authenticator_threshold"></a>

## Function `threshold`

Return the threshold for the multisig policy
<code>k</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_threshold">threshold</a>(k: &<a href="#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Authenticator_threshold">threshold</a>(k: &<a href="#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): u8 {
    *&k.threshold
}
</code></pre>



</details>
