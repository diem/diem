
<a name="0x1_Coin1"></a>

# Module `0x1::Coin1`

### Table of Contents

-  [Struct `Coin1`](#0x1_Coin1_Coin1)
-  [Function `initialize`](#0x1_Coin1_initialize)
-  [Specification](#0x1_Coin1_Specification)



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



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Coin1_initialize">initialize</a>(lr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Coin1_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Libra.md#0x1_Libra_register_SCS_currency">Libra::register_SCS_currency</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(
        lr_account,
        tc_account,
        <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2), // exchange rate <b>to</b> <a href="LBR.md#0x1_LBR">LBR</a>
        1000000, // scaling_factor = 10^6
        100,     // fractional_part = 10^2
        b"<a href="#0x1_Coin1">Coin1</a>"
    );
    <a href="AccountLimits.md#0x1_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(lr_account);
}
</code></pre>



</details>

<a name="0x1_Coin1_Specification"></a>

## Specification



<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="Libra.md#0x1_Libra_is_currency">Libra::is_currency</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;();
</code></pre>
