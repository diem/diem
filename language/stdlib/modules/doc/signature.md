
<a name="0x0_Signature"></a>

# Module `0x0::Signature`

### Table of Contents

-  [Function `ed25519_validate_pubkey`](#0x0_Signature_ed25519_validate_pubkey)
-  [Function `ed25519_verify`](#0x0_Signature_ed25519_verify)
-  [Function `ed25519_threshold_verify`](#0x0_Signature_ed25519_threshold_verify)



<a name="0x0_Signature_ed25519_validate_pubkey"></a>

## Function `ed25519_validate_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Signature_ed25519_validate_pubkey">ed25519_validate_pubkey</a>(public_key: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Signature_ed25519_validate_pubkey">ed25519_validate_pubkey</a>(public_key: vector&lt;u8&gt;): bool;
</code></pre>



</details>

<a name="0x0_Signature_ed25519_verify"></a>

## Function `ed25519_verify`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Signature_ed25519_verify">ed25519_verify</a>(signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Signature_ed25519_verify">ed25519_verify</a>(signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): bool;
</code></pre>



</details>

<a name="0x0_Signature_ed25519_threshold_verify"></a>

## Function `ed25519_threshold_verify`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Signature_ed25519_threshold_verify">ed25519_threshold_verify</a>(bitmap: vector&lt;u8&gt;, signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Signature_ed25519_threshold_verify">ed25519_threshold_verify</a>(bitmap: vector&lt;u8&gt;, signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): u64;
</code></pre>



</details>
