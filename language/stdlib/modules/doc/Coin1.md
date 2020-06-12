
<a name="0x1_Coin1"></a>

# Module `0x1::Coin1`

### Table of Contents

-  [Struct `Coin1`](#0x1_Coin1_Coin1)
-  [Function `initialize`](#0x1_Coin1_initialize)



<a name="0x1_Coin1_Coin1"></a>

## Struct `Coin1`



<pre><code><b>struct</b> <a href="#0x1_Coin1">Coin1</a>
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

<a name="0x1_Coin1_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Coin1_initialize">initialize</a>(account: &signer, register_currency_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Libra.md#0x1_Libra_RegisterNewCurrency">Libra::RegisterNewCurrency</a>&gt;): (<a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Coin1_initialize">initialize</a>(
    account: &signer,
    register_currency_capability: &Capability&lt;RegisterNewCurrency&gt;,
): (<a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;, <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;) {
    // Register the <a href="#0x1_Coin1">Coin1</a> currency.
    <a href="Libra.md#0x1_Libra_register_currency">Libra::register_currency</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(
        account,
        register_currency_capability,
        <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2), // exchange rate <b>to</b> <a href="LBR.md#0x1_LBR">LBR</a>
        <b>false</b>,   // is_synthetic
        1000000, // scaling_factor = 10^6
        100,     // fractional_part = 10^2
        b"<a href="#0x1_Coin1">Coin1</a>",
    )
}
</code></pre>



</details>
