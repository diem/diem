
<a name="0x0_LBR"></a>

# Module `0x0::LBR`

### Table of Contents

-  [Struct `T`](#0x0_LBR_T)
-  [Struct `ReserveComponent`](#0x0_LBR_ReserveComponent)
-  [Struct `Reserve`](#0x0_LBR_Reserve)
-  [Function `initialize`](#0x0_LBR_initialize)
-  [Function `swap_into`](#0x0_LBR_swap_into)
-  [Function `create`](#0x0_LBR_create)
-  [Function `unpack`](#0x0_LBR_unpack)
-  [Function `mint`](#0x0_LBR_mint)



<a name="0x0_LBR_T"></a>

## Struct `T`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LBR_T">T</a>
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

<a name="0x0_LBR_ReserveComponent"></a>

## Struct `ReserveComponent`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LBR_ReserveComponent">ReserveComponent</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>ratio: <a href="fixedpoint32.md#0x0_FixedPoint32_T">FixedPoint32::T</a></code>
</dt>
<dd>

</dd>
<dt>

<code>backing: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LBR_Reserve"></a>

## Struct `Reserve`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LBR_Reserve">Reserve</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>mint_cap: <a href="libra.md#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="#0x0_LBR_T">LBR::T</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>burn_cap: <a href="libra.md#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="#0x0_LBR_T">LBR::T</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>preburn_cap: <a href="libra.md#0x0_Libra_Preburn">Libra::Preburn</a>&lt;<a href="#0x0_LBR_T">LBR::T</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>coin1: <a href="#0x0_LBR_ReserveComponent">LBR::ReserveComponent</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>coin2: <a href="#0x0_LBR_ReserveComponent">LBR::ReserveComponent</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LBR_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LBR_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LBR_initialize">initialize</a>(account: &signer) {
    // Register the <a href="#0x0_LBR">LBR</a> currency.
    <b>let</b> (mint_cap, burn_cap) = <a href="libra.md#0x0_Libra_register_currency">Libra::register_currency</a>&lt;<a href="#0x0_LBR_T">T</a>&gt;(
        account,
        <a href="fixedpoint32.md#0x0_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 1), // exchange rate <b>to</b> <a href="#0x0_LBR">LBR</a>
        <b>true</b>,    // is_synthetic
        1000000, // scaling_factor = 10^6
        1000,    // fractional_part = 10^3
        x"4C4252" // UTF8-encoded "<a href="#0x0_LBR">LBR</a>" <b>as</b> a hex string
    );
    <b>let</b> preburn_cap = <a href="libra.md#0x0_Libra_new_preburn_with_capability">Libra::new_preburn_with_capability</a>(&burn_cap);
    <b>let</b> coin1 = <a href="#0x0_LBR_ReserveComponent">ReserveComponent</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt; {
        ratio: <a href="fixedpoint32.md#0x0_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2),
        backing: <a href="libra.md#0x0_Libra_zero">Libra::zero</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(),
    };
    <b>let</b> coin2 = <a href="#0x0_LBR_ReserveComponent">ReserveComponent</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt; {
        ratio: <a href="fixedpoint32.md#0x0_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2),
        backing: <a href="libra.md#0x0_Libra_zero">Libra::zero</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(),
    };
    move_to(account, <a href="#0x0_LBR_Reserve">Reserve</a> { mint_cap, burn_cap, preburn_cap, coin1, coin2 });
}
</code></pre>



</details>

<a name="0x0_LBR_swap_into"></a>

## Function `swap_into`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LBR_swap_into">swap_into</a>(coin1: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, coin2: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;): (<a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="#0x0_LBR_T">LBR::T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LBR_swap_into">swap_into</a>(
    coin1: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;,
    coin2: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;
): (<a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="#0x0_LBR_T">T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;)
<b>acquires</b> <a href="#0x0_LBR_Reserve">Reserve</a> {
    <b>let</b> reserve = borrow_global_mut&lt;<a href="#0x0_LBR_Reserve">Reserve</a>&gt;(0xA550C18);
    <b>let</b> coin1_value = <a href="libra.md#0x0_Libra_value">Libra::value</a>(&coin1);
    <b>let</b> coin2_value = <a href="libra.md#0x0_Libra_value">Libra::value</a>(&coin2);
    <b>if</b> (coin1_value &lt;= 1 || coin2_value &lt;= 1) <b>return</b> (<a href="libra.md#0x0_Libra_zero">Libra::zero</a>&lt;<a href="#0x0_LBR_T">T</a>&gt;(), coin1, coin2);
    <b>let</b> lbr_num_coin1 = <a href="fixedpoint32.md#0x0_FixedPoint32_divide_u64">FixedPoint32::divide_u64</a>(coin1_value - 1, *&reserve.coin1.ratio);
    <b>let</b> lbr_num_coin2 = <a href="fixedpoint32.md#0x0_FixedPoint32_divide_u64">FixedPoint32::divide_u64</a>(coin2_value - 1, *&reserve.coin2.ratio);
    <b>let</b> num_lbr = <b>if</b> (lbr_num_coin2 &lt; lbr_num_coin1) {
        lbr_num_coin2
    } <b>else</b> {
        lbr_num_coin1
    };
    <a href="#0x0_LBR_create">create</a>(num_lbr, coin1, coin2)
}
</code></pre>



</details>

<a name="0x0_LBR_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LBR_create">create</a>(amount_lbr: u64, coin1: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, coin2: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;): (<a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="#0x0_LBR_T">LBR::T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LBR_create">create</a>(
    amount_lbr: u64,
    coin1: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;,
    coin2: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;
): (<a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="#0x0_LBR_T">T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;)
<b>acquires</b> <a href="#0x0_LBR_Reserve">Reserve</a> {
    <b>if</b> (amount_lbr == 0) <b>return</b> (<a href="libra.md#0x0_Libra_zero">Libra::zero</a>&lt;<a href="#0x0_LBR_T">T</a>&gt;(), coin1, coin2);
    <b>let</b> reserve = borrow_global_mut&lt;<a href="#0x0_LBR_Reserve">Reserve</a>&gt;(0xA550C18);
    <b>let</b> num_coin1 = 1 + <a href="fixedpoint32.md#0x0_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin1.ratio);
    <b>let</b> num_coin2 = 1 + <a href="fixedpoint32.md#0x0_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin2.ratio);
    <b>let</b> coin1_exact = <a href="libra.md#0x0_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> coin1, num_coin1);
    <b>let</b> coin2_exact = <a href="libra.md#0x0_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> coin2, num_coin2);
    <a href="libra.md#0x0_Libra_deposit">Libra::deposit</a>(&<b>mut</b> reserve.coin1.backing, coin1_exact);
    <a href="libra.md#0x0_Libra_deposit">Libra::deposit</a>(&<b>mut</b> reserve.coin2.backing, coin2_exact);
    (<a href="libra.md#0x0_Libra_mint_with_capability">Libra::mint_with_capability</a>&lt;<a href="#0x0_LBR_T">T</a>&gt;(amount_lbr, &reserve.mint_cap), coin1, coin2)
}
</code></pre>



</details>

<a name="0x0_LBR_unpack"></a>

## Function `unpack`



<pre><code><b>public</b> <b>fun</b> <b>unpack</b>(coin: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="#0x0_LBR_T">LBR::T</a>&gt;): (<a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <b>unpack</b>(coin: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="#0x0_LBR_T">T</a>&gt;): (<a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;)
<b>acquires</b> <a href="#0x0_LBR_Reserve">Reserve</a> {
    <b>let</b> reserve = borrow_global_mut&lt;<a href="#0x0_LBR_Reserve">Reserve</a>&gt;(0xA550C18);
    <b>let</b> ratio_multiplier = <a href="libra.md#0x0_Libra_value">Libra::value</a>(&coin);
    <b>let</b> sender = Transaction::sender();
    <a href="libra.md#0x0_Libra_preburn_with_resource">Libra::preburn_with_resource</a>(coin, &<b>mut</b> reserve.preburn_cap, sender);
    <a href="libra.md#0x0_Libra_burn_with_resource_cap">Libra::burn_with_resource_cap</a>(&<b>mut</b> reserve.preburn_cap, sender, &reserve.burn_cap);
    <b>let</b> coin1_amount = <a href="fixedpoint32.md#0x0_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(ratio_multiplier, *&reserve.coin1.ratio);
    <b>let</b> coin2_amount = <a href="fixedpoint32.md#0x0_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(ratio_multiplier, *&reserve.coin2.ratio);
    <b>let</b> coin1 = <a href="libra.md#0x0_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> reserve.coin1.backing, coin1_amount);
    <b>let</b> coin2 = <a href="libra.md#0x0_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> reserve.coin2.backing, coin2_amount);
    (coin1, coin2)
}
</code></pre>



</details>

<a name="0x0_LBR_mint"></a>

## Function `mint`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LBR_mint">mint</a>(amount_lbr: u64): <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="#0x0_LBR_T">LBR::T</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LBR_mint">mint</a>(amount_lbr: u64): <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;<a href="#0x0_LBR_T">T</a>&gt; <b>acquires</b> <a href="#0x0_LBR_Reserve">Reserve</a> {
    <b>let</b> reserve = borrow_global&lt;<a href="#0x0_LBR_Reserve">Reserve</a>&gt;(0xA550C18);
    <b>let</b> num_coin1 = 1 + <a href="fixedpoint32.md#0x0_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin1.ratio);
    <b>let</b> num_coin2 = 1 + <a href="fixedpoint32.md#0x0_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin2.ratio);
    <b>let</b> coin1 = <a href="libra.md#0x0_Libra_mint">Libra::mint</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(num_coin1);
    <b>let</b> coin2 = <a href="libra.md#0x0_Libra_mint">Libra::mint</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(num_coin2);
    <b>let</b> (lbr, leftover1, leftover2) = <a href="#0x0_LBR_create">create</a>(amount_lbr, coin1, coin2);
    <a href="libra.md#0x0_Libra_destroy_zero">Libra::destroy_zero</a>(leftover1);
    <a href="libra.md#0x0_Libra_destroy_zero">Libra::destroy_zero</a>(leftover2);
    lbr
}
</code></pre>



</details>
