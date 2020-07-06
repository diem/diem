
<a name="0x1_Signature"></a>

# Module `0x1::Signature`

### Table of Contents

-  [Function `ed25519_validate_pubkey`](#0x1_Signature_ed25519_validate_pubkey)
-  [Function `ed25519_verify`](#0x1_Signature_ed25519_verify)
-  [Specification](#0x1_Signature_Specification)

Contains functions for [ed25519](https://en.wikipedia.org/wiki/EdDSA) digital signatures.


<a name="0x1_Signature_ed25519_validate_pubkey"></a>

## Function `ed25519_validate_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Signature_ed25519_validate_pubkey">ed25519_validate_pubkey</a>(public_key: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x1_Signature_ed25519_validate_pubkey">ed25519_validate_pubkey</a>(public_key: vector&lt;u8&gt;): bool;
</code></pre>



</details>

<a name="0x1_Signature_ed25519_verify"></a>

## Function `ed25519_verify`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Signature_ed25519_verify">ed25519_verify</a>(signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x1_Signature_ed25519_verify">ed25519_verify</a>(signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): bool;
</code></pre>



</details>

<a name="0x1_Signature_Specification"></a>

## Specification

The signature verification functions are treated as uninterpreted. The uninterpreted semantics of
the Move functions aligns with that of the specification functions below.


<a name="0x1_Signature_spec_ed25519_validate_pubkey"></a>


<pre><code><b>native</b> <b>define</b> <a href="#0x1_Signature_spec_ed25519_validate_pubkey">spec_ed25519_validate_pubkey</a>(public_key: vector&lt;u8&gt;): bool;
<a name="0x1_Signature_spec_ed25519_verify"></a>
<b>native</b> <b>define</b> <a href="#0x1_Signature_spec_ed25519_verify">spec_ed25519_verify</a>(signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): bool;
</code></pre>
