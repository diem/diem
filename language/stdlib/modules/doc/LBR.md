
<a name="0x1_LBR"></a>

# Module `0x1::LBR`

NB: This module is a stub of the <code><a href="LBR.md#0x1_LBR">LBR</a></code> at the moment.

Once the component makeup of the LBR has been chosen the
<code><a href="LBR.md#0x1_LBR_Reserve">Reserve</a></code> will be updated to hold the backing coins in the correct ratios.


-  [Resource <code><a href="LBR.md#0x1_LBR">LBR</a></code>](#0x1_LBR_LBR)
-  [Resource <code><a href="LBR.md#0x1_LBR_Reserve">Reserve</a></code>](#0x1_LBR_Reserve)
-  [Const <code><a href="LBR.md#0x1_LBR_ERESERVE">ERESERVE</a></code>](#0x1_LBR_ERESERVE)
-  [Function <code>initialize</code>](#0x1_LBR_initialize)
-  [Function <code>is_lbr</code>](#0x1_LBR_is_lbr)
-  [Function <code>reserve_address</code>](#0x1_LBR_reserve_address)
-  [Module Specification](#@Module_Specification_0)


<a name="0x1_LBR_LBR"></a>

## Resource `LBR`

The type tag representing the <code><a href="LBR.md#0x1_LBR">LBR</a></code> currency on-chain.


<pre><code><b>resource</b> <b>struct</b> <a href="LBR.md#0x1_LBR">LBR</a>
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

<a name="0x1_LBR_Reserve"></a>

## Resource `Reserve`

Note: Currently only holds the mint, burn, and preburn capabilities for
LBR. Once the makeup of the LBR has been determined this resource will
be updated to hold the backing LBR reserve compnents on-chain.

The on-chain reserve for the <code><a href="LBR.md#0x1_LBR">LBR</a></code> holds both the capability for minting <code><a href="LBR.md#0x1_LBR">LBR</a></code>
coins, and also each reserve component that holds the backing for these coins on-chain.
Currently this holds no coins since LBR is not able to be minted/created.


<pre><code><b>resource</b> <b>struct</b> <a href="LBR.md#0x1_LBR_Reserve">Reserve</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_cap: <a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;</code>
</dt>
<dd>
 The mint capability allowing minting of <code><a href="LBR.md#0x1_LBR">LBR</a></code> coins.
</dd>
<dt>
<code>burn_cap: <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;</code>
</dt>
<dd>
 The burn capability for <code><a href="LBR.md#0x1_LBR">LBR</a></code> coins. This is used for the unpacking
 of <code><a href="LBR.md#0x1_LBR">LBR</a></code> coins into the underlying backing currencies.
</dd>
<dt>
<code>preburn_cap: <a href="Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;<a href="LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;</code>
</dt>
<dd>
 The preburn for <code><a href="LBR.md#0x1_LBR">LBR</a></code>. This is an administrative field since we
 need to alway preburn before we burn.
</dd>
</dl>


</details>

<a name="0x1_LBR_ERESERVE"></a>

## Const `ERESERVE`

The <code><a href="LBR.md#0x1_LBR_Reserve">Reserve</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="LBR.md#0x1_LBR_ERESERVE">ERESERVE</a>: u64 = 0;
</code></pre>



<a name="0x1_LBR_initialize"></a>

## Function `initialize`

Initializes the <code><a href="LBR.md#0x1_LBR">LBR</a></code> module. This sets up the initial <code><a href="LBR.md#0x1_LBR">LBR</a></code> ratios and
reserve components, and creates the mint, preburn, and burn
capabilities for <code><a href="LBR.md#0x1_LBR">LBR</a></code> coins. The <code><a href="LBR.md#0x1_LBR">LBR</a></code> currency must not already be
registered in order for this to succeed. The sender must both be the
correct address (<code><a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a></code>) and have the
correct permissions (<code>&Capability&lt;RegisterNewCurrency&gt;</code>). Both of these
restrictions are enforced in the <code><a href="Libra.md#0x1_Libra_register_currency">Libra::register_currency</a></code> function, but also enforced here.


<pre><code><b>public</b> <b>fun</b> <a href="LBR.md#0x1_LBR_initialize">initialize</a>(lr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LBR.md#0x1_LBR_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    // Operational constraint
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_currency_info">CoreAddresses::assert_currency_info</a>(lr_account);
    // <a href="LBR.md#0x1_LBR_Reserve">Reserve</a> must not exist.
    <b>assert</b>(!<b>exists</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="LBR.md#0x1_LBR_ERESERVE">ERESERVE</a>));
    <b>let</b> (mint_cap, burn_cap) = <a href="Libra.md#0x1_Libra_register_currency">Libra::register_currency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(
        lr_account,
        <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 1), // exchange rate <b>to</b> <a href="LBR.md#0x1_LBR">LBR</a>
        <b>true</b>,    // is_synthetic
        1000000, // scaling_factor = 10^6
        1000,    // fractional_part = 10^3
        b"<a href="LBR.md#0x1_LBR">LBR</a>"
    );
    // <a href="LBR.md#0x1_LBR">LBR</a> cannot be minted.
    <a href="Libra.md#0x1_Libra_update_minting_ability">Libra::update_minting_ability</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(tc_account, <b>false</b>);
    <a href="AccountLimits.md#0x1_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(lr_account);
    <b>let</b> preburn_cap = <a href="Libra.md#0x1_Libra_create_preburn">Libra::create_preburn</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(tc_account);
    move_to(lr_account, <a href="LBR.md#0x1_LBR_Reserve">Reserve</a> { mint_cap, burn_cap, preburn_cap });
}
</code></pre>



</details>

<a name="0x1_LBR_is_lbr"></a>

## Function `is_lbr`

Returns true if <code>CoinType</code> is <code><a href="LBR.md#0x1_LBR_LBR">LBR::LBR</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="LBR.md#0x1_LBR_is_lbr">is_lbr</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LBR.md#0x1_LBR_is_lbr">is_lbr</a>&lt;CoinType&gt;(): bool {
    <a href="Libra.md#0x1_Libra_is_currency">Libra::is_currency</a>&lt;CoinType&gt;() &&
        <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;CoinType&gt;() == <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;()
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque, verify = <b>false</b>;
<b>include</b> <a href="Libra.md#0x1_Libra_spec_is_currency">Libra::spec_is_currency</a>&lt;CoinType&gt;() ==&gt; <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;;
</code></pre>


The following is correct because currency codes are unique; however, we
can currently not prove it, therefore verify is false.


<pre><code><b>ensures</b> result == <a href="Libra.md#0x1_Libra_spec_is_currency">Libra::spec_is_currency</a>&lt;CoinType&gt;() && <a href="LBR.md#0x1_LBR_spec_is_lbr">spec_is_lbr</a>&lt;CoinType&gt;();
</code></pre>


Returns true if CoinType is LBR.


<a name="0x1_LBR_spec_is_lbr"></a>


<pre><code><b>define</b> <a href="LBR.md#0x1_LBR_spec_is_lbr">spec_is_lbr</a>&lt;CoinType&gt;(): bool {
   type&lt;CoinType&gt;() == type&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;()
}
</code></pre>



</details>

<a name="0x1_LBR_reserve_address"></a>

## Function `reserve_address`

Return the account address where the globally unique LBR::Reserve resource is stored


<pre><code><b>public</b> <b>fun</b> <a href="LBR.md#0x1_LBR_reserve_address">reserve_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LBR.md#0x1_LBR_reserve_address">reserve_address</a>(): address {
    <a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification

Global invariant that the Reserve resource exists after genesis.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="LBR.md#0x1_LBR_reserve_exists">reserve_exists</a>() && <a href="Libra.md#0x1_Libra_is_currency">Libra::is_currency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;();
<a name="0x1_LBR_reserve_exists"></a>
<b>define</b> <a href="LBR.md#0x1_LBR_reserve_exists">reserve_exists</a>(): bool {
   <b>exists</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
