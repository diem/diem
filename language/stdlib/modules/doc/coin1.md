
<a name="0x0_Coin1"></a>

# Module `0x0::Coin1`

### Table of Contents

-  [Struct `T`](#0x0_Coin1_T)
-  [Function `initialize`](#0x0_Coin1_initialize)



<a name="0x0_Coin1_T"></a>

## Struct `T`



<pre><code><b>struct</b> <a href="#0x0_Coin1_T">T</a>
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

<a name="0x0_Coin1_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Coin1_initialize">initialize</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Coin1_initialize">initialize</a>() {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    // Register the <a href="lbr.md#0x0_LBR">LBR</a> currency.
    <a href="libra.md#0x0_Libra_register_currency">Libra::register_currency</a>&lt;<a href="#0x0_Coin1_T">T</a>&gt;(
        <a href="fixedpoint32.md#0x0_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2), // exchange rate <b>to</b> <a href="lbr.md#0x0_LBR">LBR</a>
        <b>false</b>,   // is_synthetic
        1000000, // scaling_factor = 10^6
        100,     // fractional_part = 10^2
        x"436F696E31", // UTF8 encoding of "<a href="#0x0_Coin1">Coin1</a>" in hex
    );
}
</code></pre>



</details>
