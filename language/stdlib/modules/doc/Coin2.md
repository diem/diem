
<a name="0x1_Coin2"></a>

# Module `0x1::Coin2`

### Table of Contents

-  [Struct `Coin2`](#0x1_Coin2_Coin2)
-  [Function `initialize`](#0x1_Coin2_initialize)



<a name="0x1_Coin2_Coin2"></a>

## Struct `Coin2`



<pre><code><b>struct</b> <a href="#0x1_Coin2">Coin2</a>
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

<a name="0x1_Coin2_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Coin2_initialize">initialize</a>(account: &signer): (<a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;, <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Coin2_initialize">initialize</a>(account: &signer): (<a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="#0x1_Coin2">Coin2</a>&gt;, <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="#0x1_Coin2">Coin2</a>&gt;) {
    <a href="Association.md#0x1_Association_assert_is_association">Association::assert_is_association</a>(account);
    // Register the <a href="#0x1_Coin2">Coin2</a> currency.
    <a href="Libra.md#0x1_Libra_register_currency">Libra::register_currency</a>&lt;<a href="#0x1_Coin2">Coin2</a>&gt;(
        account,
        <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2), // exchange rate <b>to</b> <a href="LBR.md#0x1_LBR">LBR</a>
        <b>false</b>,   // is_synthetic
        1000000, // scaling_factor = 10^6
        100,     // fractional_part = 10^2
        b"<a href="#0x1_Coin2">Coin2</a>",
    )
}
</code></pre>



</details>
