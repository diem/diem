
<a name="0x1_LBR"></a>

# Module `0x1::LBR`

### Table of Contents

-  [Resource `LBR`](#0x1_LBR_LBR)
-  [Resource `ReserveComponent`](#0x1_LBR_ReserveComponent)
-  [Resource `Reserve`](#0x1_LBR_Reserve)
-  [Function `initialize`](#0x1_LBR_initialize)
-  [Function `is_lbr`](#0x1_LBR_is_lbr)
-  [Function `calculate_component_amounts_for_lbr`](#0x1_LBR_calculate_component_amounts_for_lbr)
-  [Function `create`](#0x1_LBR_create)
-  [Function `unpack`](#0x1_LBR_unpack)
-  [Function `reserve_address`](#0x1_LBR_reserve_address)
-  [Specification](#0x1_LBR_Specification)
    -  [Function `is_lbr`](#0x1_LBR_Specification_is_lbr)
    -  [Function `unpack`](#0x1_LBR_Specification_unpack)

This module defines the
<code><a href="#0x1_LBR">LBR</a></code> currency as an on-chain reserve. The
<code><a href="#0x1_LBR">LBR</a></code> currency differs from other currencies on-chain, since anyone can
"atomically" swap into, and out-of the
<code><a href="#0x1_LBR">LBR</a></code> as long as they hold the
underlying currencies. This is done by specifying the make up of, and
holding the reserve of backing currencies for the
<code><a href="#0x1_LBR">LBR</a></code> on-chain.
Users can create
<code><a href="#0x1_LBR">LBR</a></code> coins by passing in the backing
currencies, and can likewise "unpack"
<code><a href="#0x1_LBR">LBR</a></code> to get the backing coins
for that coin. The liquidity of the reserve is enforced by the logic in
this module that ensures that the correct amount of each backing currency
is withdrawn on creation of an
<code><a href="#0x1_LBR">LBR</a></code> coin, and that only the appropriate
amount of each coin is returned when an
<code><a href="#0x1_LBR">LBR</a></code> coin is "unpacked."


<a name="0x1_LBR_LBR"></a>

## Resource `LBR`

The type tag representing the
<code><a href="#0x1_LBR">LBR</a></code> currency on-chain.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LBR">LBR</a>
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

<a name="0x1_LBR_ReserveComponent"></a>

## Resource `ReserveComponent`

A
<code><a href="#0x1_LBR_ReserveComponent">ReserveComponent</a></code> holds one part of the on-chain reserve that backs
<code><a href="#0x1_LBR">LBR</a></code> coins. Each
<code><a href="#0x1_LBR_ReserveComponent">ReserveComponent</a></code> holds both the backing currency
itself, along with the ratio of the backing currency to the
<code><a href="#0x1_LBR">LBR</a></code> coin.
For example, if
<code><a href="Coin1.md#0x1_Coin1">Coin1</a></code> makes up 1/2 of an
<code><a href="#0x1_LBR">LBR</a></code>, then the
<code>ratio</code> field would be 0.5.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LBR_ReserveComponent">ReserveComponent</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a></code>
</dt>
<dd>
 Specifies the relative ratio between the
<code>CoinType</code> and
<code><a href="#0x1_LBR">LBR</a></code> (i.e., how
 many
<code>CoinType</code>s make up one
<code><a href="#0x1_LBR">LBR</a></code>).
</dd>
<dt>

<code>backing: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;</code>
</dt>
<dd>
 Holds the
<code>CoinType</code> backing coins for the on-chain reserve.
</dd>
</dl>


</details>

<a name="0x1_LBR_Reserve"></a>

## Resource `Reserve`

The on-chain reserve for the
<code><a href="#0x1_LBR">LBR</a></code> holds both the capability for minting
<code><a href="#0x1_LBR">LBR</a></code>
coins, and also each reserve component that holds the backing for these coins on-chain.
A crucial invariant of this on-chain reserve is that for each component
<code>c_i</code>,
<code>c_i.value/c_i.ratio &gt;= <a href="#0x1_LBR">LBR</a>.market_cap</code>.
e.g., if
<code>coin1.ratio = 1/2</code> and
<code>coin2.ratio = 1/2</code> and
<code><a href="#0x1_LBR">LBR</a>.market_cap ==
100</code>, then
<code>coin1.value &gt;= 50</code>, and
<code>coin2.value &gt;= 50</code>.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LBR_Reserve">Reserve</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>mint_cap: <a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;</code>
</dt>
<dd>
 The mint capability allowing minting of
<code><a href="#0x1_LBR">LBR</a></code> coins.
</dd>
<dt>

<code>burn_cap: <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;</code>
</dt>
<dd>
 The burn capability for
<code><a href="#0x1_LBR">LBR</a></code> coins. This is used for the unpacking
 of
<code><a href="#0x1_LBR">LBR</a></code> coins into the underlying backing currencies.
</dd>
<dt>

<code>preburn_cap: <a href="Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;</code>
</dt>
<dd>
 The preburn for
<code><a href="#0x1_LBR">LBR</a></code>. This is an administrative field since we
 need to alway preburn before we burn.
</dd>
<dt>

<code>coin1: <a href="#0x1_LBR_ReserveComponent">LBR::ReserveComponent</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;</code>
</dt>
<dd>
 The
<code><a href="Coin1.md#0x1_Coin1">Coin1</a></code> reserve component, holds the backing coins and ratio
 that needs to be held for the
<code><a href="Coin1.md#0x1_Coin1">Coin1</a></code> currency.
</dd>
<dt>

<code>coin2: <a href="#0x1_LBR_ReserveComponent">LBR::ReserveComponent</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;</code>
</dt>
<dd>
 The
<code><a href="Coin2.md#0x1_Coin2">Coin2</a></code> reserve component, holds the backing coins and ratio
 that needs to be held for the
<code><a href="Coin2.md#0x1_Coin2">Coin2</a></code> currency.
</dd>
</dl>


</details>

<a name="0x1_LBR_initialize"></a>

## Function `initialize`

Initializes the
<code><a href="#0x1_LBR">LBR</a></code> module. This sets up the initial
<code><a href="#0x1_LBR">LBR</a></code> ratios and
reserve components, and creates the mint, preburn, and burn
capabilities for
<code><a href="#0x1_LBR">LBR</a></code> coins. The
<code><a href="#0x1_LBR">LBR</a></code> currency must not already be
registered in order for this to succeed. The sender must both be the
correct address (
<code><a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a></code>) and have the
correct permissions (
<code>&Capability&lt;RegisterNewCurrency&gt;</code>). Both of these
restrictions are enforced in the
<code><a href="Libra.md#0x1_Libra_register_currency">Libra::register_currency</a></code> function, but also enforced here.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_initialize">initialize</a>(lr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
) {
    // Operational constraint
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account) == <a href="#0x1_LBR_reserve_address">reserve_address</a>(), EINVALID_SINGLETON_ADDRESS);
    // Register the `<a href="#0x1_LBR">LBR</a>` currency.
    <b>let</b> (mint_cap, burn_cap) = <a href="Libra.md#0x1_Libra_register_currency">Libra::register_currency</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;(
        lr_account,
        <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 1), // exchange rate <b>to</b> <a href="#0x1_LBR">LBR</a>
        <b>true</b>,    // is_synthetic
        1000000, // scaling_factor = 10^6
        1000,    // fractional_part = 10^3
        b"<a href="#0x1_LBR">LBR</a>"
    );
    <a href="AccountLimits.md#0x1_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;(lr_account);
    <b>let</b> preburn_cap = <a href="Libra.md#0x1_Libra_create_preburn">Libra::create_preburn</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;(tc_account);
    <b>let</b> coin1 = <a href="#0x1_LBR_ReserveComponent">ReserveComponent</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt; {
        ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2),
        backing: <a href="Libra.md#0x1_Libra_zero">Libra::zero</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(),
    };
    <b>let</b> coin2 = <a href="#0x1_LBR_ReserveComponent">ReserveComponent</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt; {
        ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2),
        backing: <a href="Libra.md#0x1_Libra_zero">Libra::zero</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(),
    };
    move_to(lr_account, <a href="#0x1_LBR_Reserve">Reserve</a> { mint_cap, burn_cap, preburn_cap, coin1, coin2 });
}
</code></pre>



</details>

<a name="0x1_LBR_is_lbr"></a>

## Function `is_lbr`

Returns true if
<code>CoinType</code> is
<code><a href="#0x1_LBR_LBR">LBR::LBR</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_is_lbr">is_lbr</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_is_lbr">is_lbr</a>&lt;CoinType&gt;(): bool {
    <a href="Libra.md#0x1_Libra_is_currency">Libra::is_currency</a>&lt;CoinType&gt;() &&
        <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;CoinType&gt;() == <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;()
}
</code></pre>



</details>

<a name="0x1_LBR_calculate_component_amounts_for_lbr"></a>

## Function `calculate_component_amounts_for_lbr`

We take the truncated multiplication + 1 (not ceiling!) to withdraw for each currency that makes up the
<code><a href="#0x1_LBR">LBR</a></code>.
We do this to ensure that the reserve is always positive. We could do this with other more complex methods such as
banker's rounding, but this adds considerable arithmetic complexity.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_calculate_component_amounts_for_lbr">calculate_component_amounts_for_lbr</a>(amount_lbr: u64): (u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_calculate_component_amounts_for_lbr">calculate_component_amounts_for_lbr</a>(amount_lbr: u64): (u64, u64)
<b>acquires</b> <a href="#0x1_LBR_Reserve">Reserve</a> {
    <b>let</b> reserve = borrow_global&lt;<a href="#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <b>let</b> num_coin1 = 1 + <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin1.ratio);
    <b>let</b> num_coin2 = 1 + <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin2.ratio);
    (num_coin1, num_coin2)
}
</code></pre>



</details>

<a name="0x1_LBR_create"></a>

## Function `create`

Create
<code>amount_lbr</code> number of
<code><a href="#0x1_LBR">LBR</a></code> from the passed in coins. If
enough of each coin is passed in, this will return the
<code><a href="#0x1_LBR">LBR</a></code>.
* If the passed in coins are not the exact amount needed to mint
<code>amount_lbr</code> LBR, the function will abort.
* If any of the coins passed-in do not hold a large enough balance--which is calculated as
<code>truncate(amount_lbr * reserve_component_c_i.ratio) + 1</code> for each coin
<code>c_i</code> passed in--the function will abort.
* If
<code>amount_lbr</code> is zero the function will abort.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_create">create</a>(amount_lbr: u64, coin1: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, coin2: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_create">create</a>(
    amount_lbr: u64,
    coin1: <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;,
    coin2: <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;
): <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;
<b>acquires</b> <a href="#0x1_LBR_Reserve">Reserve</a> {
    <b>assert</b>(amount_lbr &gt; 0, EZERO_LBR_MINT_NOT_ALLOWED);
    <b>let</b> (num_coin1, num_coin2) = <a href="#0x1_LBR_calculate_component_amounts_for_lbr">calculate_component_amounts_for_lbr</a>(amount_lbr);
    <b>let</b> reserve = borrow_global_mut&lt;<a href="#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <b>assert</b>(num_coin1 == <a href="Libra.md#0x1_Libra_value">Libra::value</a>(&coin1), ECOIN1_INVALID_AMOUNT);
    <b>assert</b>(num_coin2 == <a href="Libra.md#0x1_Libra_value">Libra::value</a>(&coin2), ECOIN2_INVALID_AMOUNT);
    // Deposit the coins in <b>to</b> the reserve
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> reserve.coin1.backing, coin1);
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> reserve.coin2.backing, coin2);
    // Once the coins have been deposited in the reserve, we can mint the <a href="#0x1_LBR">LBR</a>
    <a href="Libra.md#0x1_Libra_mint_with_capability">Libra::mint_with_capability</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;(amount_lbr, &reserve.mint_cap)
}
</code></pre>



</details>

<a name="0x1_LBR_unpack"></a>

## Function `unpack`

Unpacks an
<code><a href="#0x1_LBR">LBR</a></code> coin, and returns the backing coins that make up the
coin based upon the ratios defined for each
<code><a href="#0x1_LBR_ReserveComponent">ReserveComponent</a></code> in the
<code><a href="#0x1_LBR_Reserve">Reserve</a></code> resource. The value of each constituent coin that is
returned is the truncated value of the coin to the nearest base
currency unit w.r.t. to the
<code><a href="#0x1_LBR_ReserveComponent">ReserveComponent</a></code> ratio for the currency of
the coin and the value of
<code>coin</code>. e.g.,, if
<code>coin = 10</code> and
<code><a href="#0x1_LBR">LBR</a></code> is
defined as
<code>2/3</code>
<code><a href="Coin1.md#0x1_Coin1">Coin1</a></code> and
<code>1/3</code>
<code><a href="Coin2.md#0x1_Coin2">Coin2</a></code>, then the values returned
would be
<code>6</code> and
<code>3</code> for
<code><a href="Coin1.md#0x1_Coin1">Coin1</a></code> and
<code><a href="Coin2.md#0x1_Coin2">Coin2</a></code> respectively.


<pre><code><b>public</b> <b>fun</b> <b>unpack</b>(coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;): (<a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <b>unpack</b>(coin: <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;): (<a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;, <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;)
<b>acquires</b> <a href="#0x1_LBR_Reserve">Reserve</a> {
    <b>let</b> reserve = borrow_global_mut&lt;<a href="#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="#0x1_LBR_reserve_address">reserve_address</a>());
    <b>let</b> ratio_multiplier = <a href="Libra.md#0x1_Libra_value">Libra::value</a>(&coin);
    <b>let</b> sender = <a href="#0x1_LBR_reserve_address">reserve_address</a>();
    <a href="Libra.md#0x1_Libra_preburn_with_resource">Libra::preburn_with_resource</a>(coin, &<b>mut</b> reserve.preburn_cap, sender);
    <a href="Libra.md#0x1_Libra_burn_with_resource_cap">Libra::burn_with_resource_cap</a>(&<b>mut</b> reserve.preburn_cap, sender, &reserve.burn_cap);
    <b>let</b> coin1_amount = <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(ratio_multiplier, *&reserve.coin1.ratio);
    <b>let</b> coin2_amount = <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(ratio_multiplier, *&reserve.coin2.ratio);
    <b>let</b> coin1 = <a href="Libra.md#0x1_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> reserve.coin1.backing, coin1_amount);
    <b>let</b> coin2 = <a href="Libra.md#0x1_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> reserve.coin2.backing, coin2_amount);
    (coin1, coin2)
}
</code></pre>



</details>

<a name="0x1_LBR_reserve_address"></a>

## Function `reserve_address`

Return the account address where the globally unique LBR::Reserve resource is stored


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_reserve_address">reserve_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_reserve_address">reserve_address</a>(): address {
    <a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()
}
</code></pre>



</details>

<a name="0x1_LBR_Specification"></a>

## Specification


Returns true if the Reserve has been initialized.


<a name="0x1_LBR_spec_is_initialized"></a>


<pre><code><b>define</b> <a href="#0x1_LBR_spec_is_initialized">spec_is_initialized</a>(): bool {
    exists&lt;<a href="#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_CURRENCY_INFO_ADDRESS">CoreAddresses::SPEC_CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>



<a name="0x1_LBR_Specification_is_lbr"></a>

### Function `is_lbr`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_is_lbr">is_lbr</a>&lt;CoinType&gt;(): bool
</code></pre>




<pre><code>pragma verify = <b>false</b>, opaque = <b>true</b>;
</code></pre>


The following is correct because currency codes are unique.


<pre><code><b>ensures</b> result == <a href="#0x1_LBR_spec_is_lbr">spec_is_lbr</a>&lt;CoinType&gt;();
</code></pre>



Returns true if CoinType is LBR.


<a name="0x1_LBR_spec_is_lbr"></a>


<pre><code><b>define</b> <a href="#0x1_LBR_spec_is_lbr">spec_is_lbr</a>&lt;CoinType&gt;(): bool {
    type&lt;CoinType&gt;() == type&lt;<a href="#0x1_LBR">LBR</a>&gt;()
}
</code></pre>



<a name="0x1_LBR_Specification_unpack"></a>

### Function `unpack`


<pre><code><b>public</b> <b>fun</b> <b>unpack</b>(coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;): (<a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;)
</code></pre>



> TODO(emmazzz): turn opaque off when we are able to fully specify unpack.


<pre><code>pragma opaque = <b>true</b>;
<b>ensures</b> <a href="Libra.md#0x1_Libra_spec_market_cap">Libra::spec_market_cap</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;() == <b>old</b>(<a href="Libra.md#0x1_Libra_spec_market_cap">Libra::spec_market_cap</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;()) - coin.value;
</code></pre>
