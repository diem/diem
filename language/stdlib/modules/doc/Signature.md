
<a name="0x1_Signature"></a>

# Module `0x1::Signature`

Contains functions for [ed25519](https://en.wikipedia.org/wiki/EdDSA) digital signatures.


-  [Function <code>ed25519_validate_pubkey</code>](#0x1_Signature_ed25519_validate_pubkey)
-  [Function <code>ed25519_verify</code>](#0x1_Signature_ed25519_verify)


<a name="0x1_Signature_ed25519_validate_pubkey"></a>

## Function `ed25519_validate_pubkey`

Return <code><b>true</b></code> if the bytes in <code>public_key</code> can be parsed as a valid Ed25519 public key.
Returns <code><b>false</b></code> if <code>public_key</code> is not 32 bytes OR is 32 bytes, but does not pass
points-on-curve or small subgroup checks. See the Rust <code>libra_crypto::Ed25519PublicKey</code> type
for more details.
Does not abort.


<pre><code><b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">ed25519_validate_pubkey</a>(public_key: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">ed25519_validate_pubkey</a>(public_key: vector&lt;u8&gt;): bool;
</code></pre>



</details>

<a name="0x1_Signature_ed25519_verify"></a>

## Function `ed25519_verify`

Return true if the Ed25519 <code>signature</code> on <code>message</code> verifies against the Ed25519 public key
<code>public_key</code>.
Returns <code><b>false</b></code> if:
- <code>signature</code> is not 64 bytes
- <code>public_key</code> is not 32 bytes
- <code>public_key</code> does not pass points-on-curve or small subgroup checks,
- <code>signature and </code>public_key<code> are valid, but the signature on </code>message` does not verify.
Does not abort.


<pre><code><b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ed25519_verify">ed25519_verify</a>(signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ed25519_verify">ed25519_verify</a>(
    signature: vector&lt;u8&gt;,
    public_key: vector&lt;u8&gt;,
    message: vector&lt;u8&gt;
): bool;
</code></pre>



</details>
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
