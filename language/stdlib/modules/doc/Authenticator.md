
<a name="0x1_Authenticator"></a>

# Module `0x1::Authenticator`

Move representation of the authenticator types used in Libra:
- Ed25519 (single-sig)
- MultiEd25519 (K-of-N multisig)


-  [Struct <code><a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a></code>](#0x1_Authenticator_MultiEd25519PublicKey)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_SINGLE_ED25519_SCHEME_ID">SINGLE_ED25519_SCHEME_ID</a></code>](#0x1_Authenticator_SINGLE_ED25519_SCHEME_ID)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_MULTI_ED25519_SCHEME_ID">MULTI_ED25519_SCHEME_ID</a></code>](#0x1_Authenticator_MULTI_ED25519_SCHEME_ID)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_MAX_MULTI_ED25519_KEYS">MAX_MULTI_ED25519_KEYS</a></code>](#0x1_Authenticator_MAX_MULTI_ED25519_KEYS)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_EZERO_THRESHOLD">EZERO_THRESHOLD</a></code>](#0x1_Authenticator_EZERO_THRESHOLD)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD">ENOT_ENOUGH_KEYS_FOR_THRESHOLD</a></code>](#0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD">ENUM_KEYS_ABOVE_MAX_THRESHOLD</a></code>](#0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD)
-  [Function <code>create_multi_ed25519</code>](#0x1_Authenticator_create_multi_ed25519)
-  [Function <code>ed25519_authentication_key</code>](#0x1_Authenticator_ed25519_authentication_key)
-  [Function <code>multi_ed25519_authentication_key</code>](#0x1_Authenticator_multi_ed25519_authentication_key)
-  [Function <code>public_keys</code>](#0x1_Authenticator_public_keys)
-  [Function <code>threshold</code>](#0x1_Authenticator_threshold)
-  [Module Specification](#@Module_Specification_0)


<a name="0x1_Authenticator_MultiEd25519PublicKey"></a>

## Struct `MultiEd25519PublicKey`

A multi-ed25519 public key


<pre><code><b>struct</b> <a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>
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


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_SINGLE_ED25519_SCHEME_ID">SINGLE_ED25519_SCHEME_ID</a>: u8 = 0;
</code></pre>



<a name="0x1_Authenticator_MULTI_ED25519_SCHEME_ID"></a>

## Const `MULTI_ED25519_SCHEME_ID`

Scheme byte ID for multi-ed25519


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_MULTI_ED25519_SCHEME_ID">MULTI_ED25519_SCHEME_ID</a>: u8 = 1;
</code></pre>



<a name="0x1_Authenticator_MAX_MULTI_ED25519_KEYS"></a>

## Const `MAX_MULTI_ED25519_KEYS`

Maximum number of keys allowed in a MultiEd25519 public/private key


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_MAX_MULTI_ED25519_KEYS">MAX_MULTI_ED25519_KEYS</a>: u64 = 32;
</code></pre>



<a name="0x1_Authenticator_EZERO_THRESHOLD"></a>

## Const `EZERO_THRESHOLD`

Threshold provided was 0 which can't be used to create a <code>MultiEd25519</code> key


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_EZERO_THRESHOLD">EZERO_THRESHOLD</a>: u64 = 0;
</code></pre>



<a name="0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD"></a>

## Const `ENOT_ENOUGH_KEYS_FOR_THRESHOLD`

Not enough keys were provided for the specified threshold when creating an <code>MultiEd25519</code> key


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD">ENOT_ENOUGH_KEYS_FOR_THRESHOLD</a>: u64 = 1;
</code></pre>



<a name="0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD"></a>

## Const `ENUM_KEYS_ABOVE_MAX_THRESHOLD`

Too many keys were provided for the specified threshold when creating an <code>MultiEd25519</code> key


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD">ENUM_KEYS_ABOVE_MAX_THRESHOLD</a>: u64 = 2;
</code></pre>



<a name="0x1_Authenticator_create_multi_ed25519"></a>

## Function `create_multi_ed25519`

Create a a multisig policy from a vector of ed25519 public keys and a threshold.
Note: this does *not* check uniqueness of keys. Repeated keys are convenient to
encode weighted multisig policies. For example Alice AND 1 of Bob or Carol is
public_key: {alice_key, alice_key, bob_key, carol_key}, threshold: 3
Aborts if threshold is zero or bigger than the length of <code>public_keys</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_create_multi_ed25519">create_multi_ed25519</a>(public_keys: vector&lt;vector&lt;u8&gt;&gt;, threshold: u8): <a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_create_multi_ed25519">create_multi_ed25519</a>(
    public_keys: vector&lt;vector&lt;u8&gt;&gt;,
    threshold: u8
): <a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a> {
    // check theshold requirements
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&public_keys);
    <b>assert</b>(threshold != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Authenticator.md#0x1_Authenticator_EZERO_THRESHOLD">EZERO_THRESHOLD</a>));
    <b>assert</b>(
        (threshold <b>as</b> u64) &lt;= len,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Authenticator.md#0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD">ENOT_ENOUGH_KEYS_FOR_THRESHOLD</a>)
    );
    // the multied25519 signature scheme allows at most 32 keys
    <b>assert</b>(
        len &lt;= <a href="Authenticator.md#0x1_Authenticator_MAX_MULTI_ED25519_KEYS">MAX_MULTI_ED25519_KEYS</a>,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Authenticator.md#0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD">ENUM_KEYS_ABOVE_MAX_THRESHOLD</a>)
    );

    <a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a> { public_keys, threshold }
}
</code></pre>



</details>

<a name="0x1_Authenticator_ed25519_authentication_key"></a>

## Function `ed25519_authentication_key`

Compute an authentication key for the ed25519 public key <code>public_key</code>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_ed25519_authentication_key">ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_ed25519_authentication_key">ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt; {
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> public_key, <a href="Authenticator.md#0x1_Authenticator_SINGLE_ED25519_SCHEME_ID">SINGLE_ED25519_SCHEME_ID</a>);
    <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(public_key)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> [abstract] result == <a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">spec_ed25519_authentication_key</a>(public_key);
</code></pre>



</details>

<a name="0x1_Authenticator_multi_ed25519_authentication_key"></a>

## Function `multi_ed25519_authentication_key`

Compute a multied25519 account authentication key for the policy <code>k</code>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_multi_ed25519_authentication_key">multi_ed25519_authentication_key</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_multi_ed25519_authentication_key">multi_ed25519_authentication_key</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): vector&lt;u8&gt; {
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
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> authentication_key_preimage, <a href="Authenticator.md#0x1_Authenticator_MULTI_ED25519_SCHEME_ID">MULTI_ED25519_SCHEME_ID</a>);
    <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(authentication_key_preimage)
}
</code></pre>



</details>

<a name="0x1_Authenticator_public_keys"></a>

## Function `public_keys`

Return the public keys involved in the multisig policy <code>k</code>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_public_keys">public_keys</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): &vector&lt;vector&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_public_keys">public_keys</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): &vector&lt;vector&lt;u8&gt;&gt; {
    &k.public_keys
}
</code></pre>



</details>

<a name="0x1_Authenticator_threshold"></a>

## Function `threshold`

Return the threshold for the multisig policy <code>k</code>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_threshold">threshold</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_threshold">threshold</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): u8 {
    *&k.threshold
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification

We use an uninterpreted function to represent the result of key construction. The actual value
does not matter for the verification of callers.


<a name="0x1_Authenticator_spec_ed25519_authentication_key"></a>


<pre><code><b>define</b> <a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">spec_ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt;;
</code></pre>
