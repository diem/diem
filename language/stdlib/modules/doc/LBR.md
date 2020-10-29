
<a name="0x1_LBR"></a>

# Module `0x1::LBR`

NB: This module is a stub of the <code><a href="LBR.md#0x1_LBR">LBR</a></code> at the moment.

Once the component makeup of the LBR has been chosen the
<code><a href="LBR.md#0x1_LBR_Reserve">Reserve</a></code> will be updated to hold the backing coins in the correct ratios.


-  [Resource `LBR`](#0x1_LBR_LBR)
-  [Resource `Reserve`](#0x1_LBR_Reserve)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_LBR_initialize)
-  [Function `is_lbr`](#0x1_LBR_is_lbr)
-  [Function `reserve_address`](#0x1_LBR_reserve_address)
-  [Module Specification](#@Module_Specification_1)
    -  [Persistence of Resources](#@Persistence_of_Resources_2)
    -  [Helper Functions](#@Helper_Functions_3)


<pre><code><b>use</b> <a href="AccountLimits.md#0x1_AccountLimits">0x1::AccountLimits</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="FixedPoint32.md#0x1_FixedPoint32">0x1::FixedPoint32</a>;
<b>use</b> <a href="Libra.md#0x1_Libra">0x1::Libra</a>;
<b>use</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp">0x1::LibraTimestamp</a>;
</code></pre>



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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_LBR_ERESERVE"></a>

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

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotCurrencyInfo">CoreAddresses::AbortsIfNotCurrencyInfo</a>{account: lr_account};
<b>aborts_if</b> <b>exists</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>include</b> <a href="Libra.md#0x1_Libra_RegisterCurrencyAbortsIf">Libra::RegisterCurrencyAbortsIf</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;{
    currency_code: b"<a href="LBR.md#0x1_LBR">LBR</a>",
    scaling_factor: 1000000
};
<b>include</b> <a href="AccountLimits.md#0x1_AccountLimits_PublishUnrestrictedLimitsAbortsIf">AccountLimits::PublishUnrestrictedLimitsAbortsIf</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;{publish_account: lr_account};
<b>include</b> <a href="Libra.md#0x1_Libra_RegisterCurrencyEnsures">Libra::RegisterCurrencyEnsures</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;;
<b>include</b> <a href="Libra.md#0x1_Libra_UpdateMintingAbilityEnsures">Libra::UpdateMintingAbilityEnsures</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;{can_mint: <b>false</b>};
<b>include</b> <a href="AccountLimits.md#0x1_AccountLimits_PublishUnrestrictedLimitsEnsures">AccountLimits::PublishUnrestrictedLimitsEnsures</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;{publish_account: lr_account};
<b>ensures</b> <b>exists</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>


Registering LBR can only be done in genesis.


<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
</code></pre>


Only the LibraRoot account can register a new currency [[H8]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
</code></pre>


Only the TreasuryCompliance role can update the <code>can_mint</code> field of CurrencyInfo [[H2]][PERMISSION].
Moreover, only the TreasuryCompliance role can create Preburn.


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
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

<a name="@Module_Specification_1"></a>

## Module Specification



<a name="@Persistence_of_Resources_2"></a>

### Persistence of Resources


After genesis, the Reserve resource exists.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="LBR.md#0x1_LBR_reserve_exists">reserve_exists</a>();
</code></pre>


After genesis, LBR is registered.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="Libra.md#0x1_Libra_is_currency">Libra::is_currency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;();
</code></pre>



<a name="@Helper_Functions_3"></a>

### Helper Functions


Checks whether the Reserve resource exists.


<a name="0x1_LBR_reserve_exists"></a>


<pre><code><b>define</b> <a href="LBR.md#0x1_LBR_reserve_exists">reserve_exists</a>(): bool {
   <b>exists</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>


Returns true if CoinType is LBR.


<a name="0x1_LBR_spec_is_lbr"></a>


<pre><code><b>define</b> <a href="LBR.md#0x1_LBR_spec_is_lbr">spec_is_lbr</a>&lt;CoinType&gt;(): bool {
    type&lt;CoinType&gt;() == type&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;()
}
</code></pre>


After genesis, <code>LimitsDefinition&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;</code> is published at Libra root. It's published by
AccountLimits::publish_unrestricted_limits, but we can't prove the condition there because
it does not hold for all types (but does hold for LBR).


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>()
    ==&gt; <b>exists</b>&lt;<a href="AccountLimits.md#0x1_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>


<code>LimitsDefinition&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;</code> is not published at any other address


<pre><code><b>invariant</b> [<b>global</b>] <b>forall</b> addr: address <b>where</b> <b>exists</b>&lt;<a href="AccountLimits.md#0x1_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(addr):
    addr == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
</code></pre>


<code><a href="LBR.md#0x1_LBR_Reserve">Reserve</a></code> is persistent


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>] <b>old</b>(<b>exists</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="LBR.md#0x1_LBR_reserve_address">reserve_address</a>()))
    ==&gt; <b>exists</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">Reserve</a>&gt;(<a href="LBR.md#0x1_LBR_reserve_address">reserve_address</a>());
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/libra/lip/blob/master/lips/lip-2.md
[ROLE]: https://github.com/libra/lip/blob/master/lips/lip-2.md#roles
[PERMISSION]: https://github.com/libra/lip/blob/master/lips/lip-2.md#permissions
