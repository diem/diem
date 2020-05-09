
<a name="0x0_Coin2"></a>

# Module `0x0::Coin2`

### Table of Contents

-  [Struct `T`](#0x0_Coin2_T)
-  [Function `initialize`](#0x0_Coin2_initialize)



<a name="0x0_Coin2_T"></a>

## Struct `T`



<pre><code><b>struct</b> <a href="#0x0_Coin2_T">T</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Coin2_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Coin2_initialize">initialize</a>(account: &signer): (<a href="libra.md#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="#0x0_Coin2_T">Coin2::T</a>&gt;, <a href="libra.md#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="#0x0_Coin2_T">Coin2::T</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Coin2_initialize">initialize</a>(account: &signer): (<a href="libra.md#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="#0x0_Coin2_T">T</a>&gt;, <a href="libra.md#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="#0x0_Coin2_T">T</a>&gt;) {
    // Register the <a href="#0x0_Coin2">Coin2</a> currency.
    <a href="libra.md#0x0_Libra_register_currency">Libra::register_currency</a>&lt;<a href="#0x0_Coin2_T">T</a>&gt;(
        account,
        <a href="fixedpoint32.md#0x0_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2), // exchange rate <b>to</b> <a href="lbr.md#0x0_LBR">LBR</a>
        <b>false</b>,   // is_synthetic
        1000000, // scaling_factor = 10^6
        100,     // fractional_part = 10^2
        x"436F696E32", // UTF8 encoding of "<a href="#0x0_Coin2">Coin2</a>" in hex
    )
}
</code></pre>



</details>
