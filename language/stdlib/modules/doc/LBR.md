
<a name="0x1_LBR"></a>

# Module `0x1::LBR`

### Table of Contents

-  [Struct `LBR`](#0x1_LBR_LBR)
-  [Struct `ReserveComponent`](#0x1_LBR_ReserveComponent)
-  [Struct `Reserve`](#0x1_LBR_Reserve)
-  [Function `initialize`](#0x1_LBR_initialize)
-  [Function `is_lbr`](#0x1_LBR_is_lbr)
-  [Function `swap_into`](#0x1_LBR_swap_into)
-  [Function `create`](#0x1_LBR_create)
-  [Function `unpack`](#0x1_LBR_unpack)
-  [Function `mint`](#0x1_LBR_mint)

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

## Struct `LBR`

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

## Struct `ReserveComponent`

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

## Struct `Reserve`

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


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_initialize">initialize</a>(association: &signer, register_currency_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Libra.md#0x1_Libra_RegisterNewCurrency">Libra::RegisterNewCurrency</a>&gt;, tc_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_initialize">initialize</a>(
    association: &signer,
    register_currency_capability: &Capability&lt;RegisterNewCurrency&gt;,
    tc_capability: &Capability&lt;TreasuryComplianceRole&gt;,
) {
    // Operational constraint
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association) == <a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>(), 0);
    // Register the `<a href="#0x1_LBR">LBR</a>` currency.
    <b>let</b> (mint_cap, burn_cap) = <a href="Libra.md#0x1_Libra_register_currency">Libra::register_currency</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;(
        association,
        register_currency_capability,
        <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 1), // exchange rate <b>to</b> <a href="#0x1_LBR">LBR</a>
        <b>true</b>,    // is_synthetic
        1000000, // scaling_factor = 10^6
        1000,    // fractional_part = 10^3
        b"<a href="#0x1_LBR">LBR</a>"
    );
    <b>let</b> preburn_cap = <a href="Libra.md#0x1_Libra_create_preburn">Libra::create_preburn</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;(tc_capability);
    <b>let</b> coin1 = <a href="#0x1_LBR_ReserveComponent">ReserveComponent</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt; {
        ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2),
        backing: <a href="Libra.md#0x1_Libra_zero">Libra::zero</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(),
    };
    <b>let</b> coin2 = <a href="#0x1_LBR_ReserveComponent">ReserveComponent</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt; {
        ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2),
        backing: <a href="Libra.md#0x1_Libra_zero">Libra::zero</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(),
    };
    move_to(association, <a href="#0x1_LBR_Reserve">Reserve</a> { mint_cap, burn_cap, preburn_cap, coin1, coin2 });
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

<a name="0x1_LBR_swap_into"></a>

## Function `swap_into`

Given the constituent coins
<code>coin1</code> and
<code>coin2</code> of the
<code><a href="#0x1_LBR">LBR</a></code>, this
function calculates the maximum amount of
<code><a href="#0x1_LBR">LBR</a></code> that can be returned
for the values of the passed-in coins with respect to the coin ratios
that are specified in the respective currency's
<code><a href="#0x1_LBR_ReserveComponent">ReserveComponent</a></code>.
Any remaining amounts in the passed-in coins are returned back out
along with any newly-created
<code><a href="#0x1_LBR">LBR</a></code> coins.
If either of the coin's values are less than or equal to 1, then the
function returns the passed-in coins along with zero
<code><a href="#0x1_LBR">LBR</a></code>.
In order to ensure that the on-chain reserve remains liquid while
minimizing complexity, the values needed to create an
<code><a href="#0x1_LBR">LBR</a></code> are the
truncated division of the passed-in coin with
<code>1</code> base currency unit added.
This is different from rounding, or ceiling; e.g.,
<code>ceil(10 /
2) = 5</code> whereas we would calculate this as
<code>trunc(10/5) + 1 = 2 + 1 = 3</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_swap_into">swap_into</a>(coin1: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, coin2: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;): (<a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;, <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_swap_into">swap_into</a>(
    coin1: <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;,
    coin2: <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;
): (<a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;, <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;, <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;)
<b>acquires</b> <a href="#0x1_LBR_Reserve">Reserve</a> {
    // Grab the reserve
    <b>let</b> reserve = borrow_global_mut&lt;<a href="#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>());
    <b>let</b> coin1_value = <a href="Libra.md#0x1_Libra_value">Libra::value</a>(&coin1);
    <b>let</b> coin2_value = <a href="Libra.md#0x1_Libra_value">Libra::value</a>(&coin2);
    // If either of the coin's values is &lt;= 1, then we don't create any <a href="#0x1_LBR">LBR</a>
    <b>if</b> (coin1_value &lt;= 1 || coin2_value &lt;= 1) <b>return</b> (<a href="Libra.md#0x1_Libra_zero">Libra::zero</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;(), coin1, coin2);
    <b>let</b> lbr_num_coin1 = <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">FixedPoint32::divide_u64</a>(coin1_value - 1, *&reserve.coin1.ratio);
    <b>let</b> lbr_num_coin2 = <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">FixedPoint32::divide_u64</a>(coin2_value - 1, *&reserve.coin2.ratio);
    // The number of `<a href="#0x1_LBR">LBR</a>` that can be minted is the minimum of the amount
    // that could be possibly minted according <b>to</b> the value of the coin
    // passed in and that coin's ratio in the reserve.
    <b>let</b> num_lbr = <b>if</b> (lbr_num_coin2 &lt; lbr_num_coin1) {
        lbr_num_coin2
    } <b>else</b> {
        lbr_num_coin1
    };
    <a href="#0x1_LBR_create">create</a>(num_lbr, coin1, coin2)
}
</code></pre>



</details>

<a name="0x1_LBR_create"></a>

## Function `create`

Create
<code>amount_lbr</code> number of
<code><a href="#0x1_LBR">LBR</a></code> from the passed in coins. If
enough of each coin is passed in, this will return the
<code><a href="#0x1_LBR">LBR</a></code> along with any
remaining balances in the passed in coins. If any of the
coins passed-in do not hold a large enough balance--which is calculated as
<code>truncate(amount_lbr * reserve_component_c_i.ratio) + 1</code> for each coin
<code>c_i</code> passed in--the function will abort.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_create">create</a>(amount_lbr: u64, coin1: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, coin2: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;): (<a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;, <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_create">create</a>(
    amount_lbr: u64,
    coin1: <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;,
    coin2: <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;
): (<a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;, <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;, <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;)
<b>acquires</b> <a href="#0x1_LBR_Reserve">Reserve</a> {
    <b>if</b> (amount_lbr == 0) <b>return</b> (<a href="Libra.md#0x1_Libra_zero">Libra::zero</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;(), coin1, coin2);
    <b>let</b> reserve = borrow_global_mut&lt;<a href="#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>());
    // We take the truncated multiplication + 1 (not ceiling!) <b>to</b> withdraw for each currency.
    // This is because we want <b>to</b> ensure that the reserve is always
    // positive. We could do this with other more complex methods such <b>as</b>
    // bankers rounding, but this adds considerable arithmetic complexity.
    <b>let</b> num_coin1 = 1 + <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin1.ratio);
    <b>let</b> num_coin2 = 1 + <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin2.ratio);
    <b>let</b> coin1_exact = <a href="Libra.md#0x1_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> coin1, num_coin1);
    <b>let</b> coin2_exact = <a href="Libra.md#0x1_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> coin2, num_coin2);
    // Deposit the coins in <b>to</b> the reserve
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> reserve.coin1.backing, coin1_exact);
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> reserve.coin2.backing, coin2_exact);
    // Once the coins have been deposited in the reserve, we can mint the <a href="#0x1_LBR">LBR</a>
    (<a href="Libra.md#0x1_Libra_mint_with_capability">Libra::mint_with_capability</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;(amount_lbr, &reserve.mint_cap), coin1, coin2)
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


<pre><code><b>public</b> <b>fun</b> <b>unpack</b>(account: &signer, coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;): (<a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <b>unpack</b>(account: &signer, coin: <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="#0x1_LBR">LBR</a>&gt;): (<a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;, <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;)
<b>acquires</b> <a href="#0x1_LBR_Reserve">Reserve</a> {
    <b>let</b> reserve = borrow_global_mut&lt;<a href="#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>());
    <b>let</b> ratio_multiplier = <a href="Libra.md#0x1_Libra_value">Libra::value</a>(&coin);
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
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

<a name="0x1_LBR_mint"></a>

## Function `mint`

Creates
<code>amount_lbr</code>
<code><a href="#0x1_LBR">LBR</a></code> using the
<code>MintCapability</code> for the backing currencies in the reserve.
Calls to this will abort if the caller does not have the appropriate
<code>MintCapability</code> for each of the backing currencies.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_mint">mint</a>(account: &signer, amount_lbr: u64): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;<a href="#0x1_LBR_LBR">LBR::LBR</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LBR_mint">mint</a>(account: &signer, amount_lbr: u64): <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="#0x1_LBR">LBR</a>&gt; <b>acquires</b> <a href="#0x1_LBR_Reserve">Reserve</a> {
    <b>let</b> reserve = borrow_global&lt;<a href="#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>());
    <b>let</b> num_coin1 = 1 + <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin1.ratio);
    <b>let</b> num_coin2 = 1 + <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(amount_lbr, *&reserve.coin2.ratio);
    <b>let</b> coin1 = <a href="Libra.md#0x1_Libra_mint">Libra::mint</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(account, num_coin1);
    <b>let</b> coin2 = <a href="Libra.md#0x1_Libra_mint">Libra::mint</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(account, num_coin2);
    <b>let</b> (lbr, leftover1, leftover2) = <a href="#0x1_LBR_create">create</a>(amount_lbr, coin1, coin2);
    <a href="Libra.md#0x1_Libra_destroy_zero">Libra::destroy_zero</a>(leftover1);
    <a href="Libra.md#0x1_Libra_destroy_zero">Libra::destroy_zero</a>(leftover2);
    lbr
}
</code></pre>



</details>
