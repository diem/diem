
<a name="0x1_Libra"></a>

# Module `0x1::Libra`

### Table of Contents

-  [Resource `RegisterNewCurrency`](#0x1_Libra_RegisterNewCurrency)
-  [Resource `Libra`](#0x1_Libra_Libra)
-  [Resource `MintCapability`](#0x1_Libra_MintCapability)
-  [Resource `BurnCapability`](#0x1_Libra_BurnCapability)
-  [Struct `MintEvent`](#0x1_Libra_MintEvent)
-  [Struct `BurnEvent`](#0x1_Libra_BurnEvent)
-  [Struct `PreburnEvent`](#0x1_Libra_PreburnEvent)
-  [Struct `CancelBurnEvent`](#0x1_Libra_CancelBurnEvent)
-  [Struct `ToLBRExchangeRateUpdateEvent`](#0x1_Libra_ToLBRExchangeRateUpdateEvent)
-  [Resource `CurrencyInfo`](#0x1_Libra_CurrencyInfo)
-  [Resource `Preburn`](#0x1_Libra_Preburn)
-  [Const `MAX_SCALING_FACTOR`](#0x1_Libra_MAX_SCALING_FACTOR)
-  [Const `MAX_U64`](#0x1_Libra_MAX_U64)
-  [Const `MAX_U128`](#0x1_Libra_MAX_U128)
-  [Const `EBURN_CAPABILITY`](#0x1_Libra_EBURN_CAPABILITY)
-  [Const `ECURRENCY_INFO`](#0x1_Libra_ECURRENCY_INFO)
-  [Const `EPREBURN`](#0x1_Libra_EPREBURN)
-  [Const `EPREBURN_OCCUPIED`](#0x1_Libra_EPREBURN_OCCUPIED)
-  [Const `EPREBURN_EMPTY`](#0x1_Libra_EPREBURN_EMPTY)
-  [Const `EMINTING_NOT_ALLOWED`](#0x1_Libra_EMINTING_NOT_ALLOWED)
-  [Const `EIS_SYNTHETIC_CURRENCY`](#0x1_Libra_EIS_SYNTHETIC_CURRENCY)
-  [Const `ECOIN`](#0x1_Libra_ECOIN)
-  [Const `EDESTRUCTION_OF_NONZERO_COIN`](#0x1_Libra_EDESTRUCTION_OF_NONZERO_COIN)
-  [Const `EMINT_CAPABILITY`](#0x1_Libra_EMINT_CAPABILITY)
-  [Const `EAMOUNT_EXCEEDS_COIN_VALUE`](#0x1_Libra_EAMOUNT_EXCEEDS_COIN_VALUE)
-  [Function `initialize`](#0x1_Libra_initialize)
-  [Function `publish_burn_capability`](#0x1_Libra_publish_burn_capability)
-  [Function `mint`](#0x1_Libra_mint)
-  [Function `burn`](#0x1_Libra_burn)
-  [Function `cancel_burn`](#0x1_Libra_cancel_burn)
-  [Function `mint_with_capability`](#0x1_Libra_mint_with_capability)
-  [Function `preburn_with_resource`](#0x1_Libra_preburn_with_resource)
-  [Function `create_preburn`](#0x1_Libra_create_preburn)
-  [Function `publish_preburn_to_account`](#0x1_Libra_publish_preburn_to_account)
-  [Function `preburn_to`](#0x1_Libra_preburn_to)
-  [Function `burn_with_capability`](#0x1_Libra_burn_with_capability)
-  [Function `burn_with_resource_cap`](#0x1_Libra_burn_with_resource_cap)
-  [Function `cancel_burn_with_capability`](#0x1_Libra_cancel_burn_with_capability)
-  [Function `burn_now`](#0x1_Libra_burn_now)
-  [Function `remove_burn_capability`](#0x1_Libra_remove_burn_capability)
-  [Function `preburn_value`](#0x1_Libra_preburn_value)
-  [Function `zero`](#0x1_Libra_zero)
-  [Function `value`](#0x1_Libra_value)
-  [Function `split`](#0x1_Libra_split)
-  [Function `withdraw`](#0x1_Libra_withdraw)
-  [Function `withdraw_all`](#0x1_Libra_withdraw_all)
-  [Function `join`](#0x1_Libra_join)
-  [Function `deposit`](#0x1_Libra_deposit)
-  [Function `destroy_zero`](#0x1_Libra_destroy_zero)
-  [Function `register_currency`](#0x1_Libra_register_currency)
-  [Function `register_SCS_currency`](#0x1_Libra_register_SCS_currency)
-  [Function `market_cap`](#0x1_Libra_market_cap)
-  [Function `approx_lbr_for_value`](#0x1_Libra_approx_lbr_for_value)
-  [Function `approx_lbr_for_coin`](#0x1_Libra_approx_lbr_for_coin)
-  [Function `is_currency`](#0x1_Libra_is_currency)
-  [Function `is_SCS_currency`](#0x1_Libra_is_SCS_currency)
-  [Function `is_synthetic_currency`](#0x1_Libra_is_synthetic_currency)
-  [Function `scaling_factor`](#0x1_Libra_scaling_factor)
-  [Function `fractional_part`](#0x1_Libra_fractional_part)
-  [Function `currency_code`](#0x1_Libra_currency_code)
-  [Function `update_lbr_exchange_rate`](#0x1_Libra_update_lbr_exchange_rate)
-  [Function `lbr_exchange_rate`](#0x1_Libra_lbr_exchange_rate)
-  [Function `update_minting_ability`](#0x1_Libra_update_minting_ability)
-  [Function `assert_is_currency`](#0x1_Libra_assert_is_currency)
-  [Function `assert_is_SCS_currency`](#0x1_Libra_assert_is_SCS_currency)
-  [Specification](#0x1_Libra_Specification)
    -  [Resource `CurrencyInfo`](#0x1_Libra_Specification_CurrencyInfo)
    -  [Function `publish_burn_capability`](#0x1_Libra_Specification_publish_burn_capability)
    -  [Function `mint`](#0x1_Libra_Specification_mint)
    -  [Function `burn`](#0x1_Libra_Specification_burn)
    -  [Function `cancel_burn`](#0x1_Libra_Specification_cancel_burn)
    -  [Function `mint_with_capability`](#0x1_Libra_Specification_mint_with_capability)
    -  [Function `preburn_with_resource`](#0x1_Libra_Specification_preburn_with_resource)
    -  [Function `publish_preburn_to_account`](#0x1_Libra_Specification_publish_preburn_to_account)
    -  [Function `preburn_to`](#0x1_Libra_Specification_preburn_to)
    -  [Function `burn_with_capability`](#0x1_Libra_Specification_burn_with_capability)
    -  [Function `burn_with_resource_cap`](#0x1_Libra_Specification_burn_with_resource_cap)
    -  [Function `burn_now`](#0x1_Libra_Specification_burn_now)
    -  [Function `remove_burn_capability`](#0x1_Libra_Specification_remove_burn_capability)
    -  [Function `split`](#0x1_Libra_Specification_split)
    -  [Function `withdraw`](#0x1_Libra_Specification_withdraw)
    -  [Function `withdraw_all`](#0x1_Libra_Specification_withdraw_all)
    -  [Function `join`](#0x1_Libra_Specification_join)
    -  [Function `deposit`](#0x1_Libra_Specification_deposit)
    -  [Function `destroy_zero`](#0x1_Libra_Specification_destroy_zero)
    -  [Function `register_currency`](#0x1_Libra_Specification_register_currency)
    -  [Function `register_SCS_currency`](#0x1_Libra_Specification_register_SCS_currency)
    -  [Function `approx_lbr_for_value`](#0x1_Libra_Specification_approx_lbr_for_value)
    -  [Function `currency_code`](#0x1_Libra_Specification_currency_code)
    -  [Function `update_lbr_exchange_rate`](#0x1_Libra_Specification_update_lbr_exchange_rate)
    -  [Function `assert_is_currency`](#0x1_Libra_Specification_assert_is_currency)
    -  [Module Specification](#0x1_Libra_@Module_Specification)
        -  [Minting](#0x1_Libra_@Minting)
        -  [Conservation of currency](#0x1_Libra_@Conservation_of_currency)

The
<code><a href="#0x1_Libra">Libra</a></code> module describes the concept of a coin in the Libra framework. It introduces the
resource
<code><a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;</code>, representing a coin of given coin type.
The module defines functions operating on coins as well as functionality like
minting and burning of coins.


<a name="0x1_Libra_RegisterNewCurrency"></a>

## Resource `RegisterNewCurrency`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Libra_RegisterNewCurrency">RegisterNewCurrency</a>
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

<a name="0x1_Libra_Libra"></a>

## Resource `Libra`

The
<code><a href="#0x1_Libra">Libra</a></code> resource defines the Libra coin for each currency in
Libra. Each "coin" is coupled with a type
<code>CoinType</code> specifying the
currency of the coin, and a
<code>value</code> field specifying the value
of the coin (in the base units of the currency
<code>CoinType</code>
and specified in the
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code> resource for that
<code>CoinType</code>
published under the
<code><a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code> account address).


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>value: u64</code>
</dt>
<dd>
 The value of this coin in the base units for
<code>CoinType</code>
</dd>
</dl>


</details>

<a name="0x1_Libra_MintCapability"></a>

## Resource `MintCapability`

The
<code><a href="#0x1_Libra_MintCapability">MintCapability</a></code> resource defines a capability to allow minting
of coins of
<code>CoinType</code> currency by the holder of this capability.
This capability is held only either by the
<code><a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()</code>
account or the
<code><a href="LBR.md#0x1_LBR">0x1::LBR</a></code> module (and
<code><a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()</code> in testnet).


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;
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

<a name="0x1_Libra_BurnCapability"></a>

## Resource `BurnCapability`

The
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a></code> resource defines a capability to allow coins
of
<code>CoinType</code> currency to be burned by the holder of the
and the
<code><a href="LBR.md#0x1_LBR">0x1::LBR</a></code> module (and
<code><a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()</code> in testnet).


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
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

<a name="0x1_Libra_MintEvent"></a>

## Struct `MintEvent`

The
<code>CurrencyRegistrationCapability</code> is a singleton resource
published under the
<code><a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()</code> and grants
the capability to the
<code><a href="#0x1_Libra">0x1::Libra</a></code> module to add currencies to the
<code><a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">0x1::RegisteredCurrencies</a></code> on-chain config.
A
<code><a href="#0x1_Libra_MintEvent">MintEvent</a></code> is emitted every time a Libra coin is minted. This
contains the
<code>amount</code> minted (in base units of the currency being
minted) along with the
<code>currency_code</code> for the coin(s) being
minted, and that is defined in the
<code>currency_code</code> field of the
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code> resource for the currency.


<pre><code><b>struct</b> <a href="#0x1_Libra_MintEvent">MintEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>
 Funds added to the system
</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 ASCII encoded symbol for the coin type (e.g., "LBR")
</dd>
</dl>


</details>

<a name="0x1_Libra_BurnEvent"></a>

## Struct `BurnEvent`

A
<code><a href="#0x1_Libra_BurnEvent">BurnEvent</a></code> is emitted every time a non-synthetic[1] Libra coin is
burned. It contains the
<code>amount</code> burned in base units for the
currency, along with the
<code>currency_code</code> for the coins being burned
(and as defined in the
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code> resource for that currency).
It also contains the
<code>preburn_address</code> from which the coin is
extracted for burning.
[1] As defined by the
<code>is_synthetic</code> field in the
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code>
for that currency.


<pre><code><b>struct</b> <a href="#0x1_Libra_BurnEvent">BurnEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>
 Funds removed from the system
</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 ASCII encoded symbol for the coin type (e.g., "LBR")
</dd>
<dt>

<code>preburn_address: address</code>
</dt>
<dd>
 Address with the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource that stored the now-burned funds
</dd>
</dl>


</details>

<a name="0x1_Libra_PreburnEvent"></a>

## Struct `PreburnEvent`

A
<code><a href="#0x1_Libra_PreburnEvent">PreburnEvent</a></code> is emitted every time an
<code>amount</code> of funds with
a coin type
<code>currency_code</code> are moved to a
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource under
the account at the address
<code>preburn_address</code>.


<pre><code><b>struct</b> <a href="#0x1_Libra_PreburnEvent">PreburnEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>
 The amount of funds waiting to be removed (burned) from the system
</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 ASCII encoded symbol for the coin type (e.g., "LBR")
</dd>
<dt>

<code>preburn_address: address</code>
</dt>
<dd>
 Address with the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource that now holds the funds
</dd>
</dl>


</details>

<a name="0x1_Libra_CancelBurnEvent"></a>

## Struct `CancelBurnEvent`

A
<code><a href="#0x1_Libra_CancelBurnEvent">CancelBurnEvent</a></code> is emitted every time funds of
<code>amount</code> in a
<code><a href="#0x1_Libra_Preburn">Preburn</a></code>
resource at
<code>preburn_address</code> is canceled (removed from the
preburn, but not burned). The currency of the funds is given by the
<code>currency_code</code> as defined in the
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code> for that currency.


<pre><code><b>struct</b> <a href="#0x1_Libra_CancelBurnEvent">CancelBurnEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>
 The amount of funds returned
</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 ASCII encoded symbol for the coin type (e.g., "LBR")
</dd>
<dt>

<code>preburn_address: address</code>
</dt>
<dd>
 Address of the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource that held the now-returned funds.
</dd>
</dl>


</details>

<a name="0x1_Libra_ToLBRExchangeRateUpdateEvent"></a>

## Struct `ToLBRExchangeRateUpdateEvent`

An
<code><a href="#0x1_Libra_ToLBRExchangeRateUpdateEvent">ToLBRExchangeRateUpdateEvent</a></code> is emitted every time the to-LBR exchange
rate for the currency given by
<code>currency_code</code> is updated.


<pre><code><b>struct</b> <a href="#0x1_Libra_ToLBRExchangeRateUpdateEvent">ToLBRExchangeRateUpdateEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 The currency code of the currency whose exchange rate was updated.
</dd>
<dt>

<code>new_to_lbr_exchange_rate: u64</code>
</dt>
<dd>
 The new on-chain to-LBR exchange rate between the
 <code>currency_code</code> currency and LBR. Represented in conversion
 between the (on-chain) base-units for the currency and microlibra.
</dd>
</dl>


</details>

<a name="0x1_Libra_CurrencyInfo"></a>

## Resource `CurrencyInfo`

The
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;</code> resource stores the various
pieces of information needed for a currency (
<code>CoinType</code>) that is
registered on-chain. This resource _must_ be published under the
address given by
<code><a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code> in order for the registration of
<code>CoinType</code> as a recognized currency on-chain to be successful. At
the time of registration the
<code><a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> and
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> capabilities are returned to the caller.
Unless they are specified otherwise the fields in this resource are immutable.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>total_value: u128</code>
</dt>
<dd>
 The total value for the currency represented by
<code>CoinType</code>. Mutable.
</dd>
<dt>

<code>preburn_value: u64</code>
</dt>
<dd>
 Value of funds that are in the process of being burned.  Mutable.
</dd>
<dt>

<code>to_lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a></code>
</dt>
<dd>
 The (rough) exchange rate from
<code>CoinType</code> to
<code><a href="LBR.md#0x1_LBR">LBR</a></code>. Mutable.
</dd>
<dt>

<code>is_synthetic: bool</code>
</dt>
<dd>
 Holds whether or not this currency is synthetic (contributes to the
 off-chain reserve) or not. An example of such a synthetic
currency would be the LBR.
</dd>
<dt>

<code>scaling_factor: u64</code>
</dt>
<dd>
 The scaling factor for the coin (i.e. the amount to multiply by
 to get to the human-readable representation for this currency).
 e.g. 10^6 for
<code><a href="Coin1.md#0x1_Coin1">Coin1</a></code>

 > TODO(wrwg): should the above be "to divide by"?
</dd>
<dt>

<code>fractional_part: u64</code>
</dt>
<dd>
 The smallest fractional part (number of decimal places) to be
 used in the human-readable representation for the currency (e.g.
 10^2 for
<code><a href="Coin1.md#0x1_Coin1">Coin1</a></code> cents)
</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 The code symbol for this
<code>CoinType</code>. ASCII encoded.
 e.g. for "LBR" this is x"4C4252". No character limit.
</dd>
<dt>

<code>can_mint: bool</code>
</dt>
<dd>
 We may want to disable the ability to mint further coins of a
 currency while that currency is still around. This allows us to
 keep the currency in circulation while disallowing further
 creation of coins in the
<code>CoinType</code> currency. Mutable.
</dd>
<dt>

<code>mint_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_MintEvent">Libra::MintEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for minting and where
<code><a href="#0x1_Libra_MintEvent">MintEvent</a></code>s will be emitted.
</dd>
<dt>

<code>burn_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_BurnEvent">Libra::BurnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for burning, and where
<code><a href="#0x1_Libra_BurnEvent">BurnEvent</a></code>s will be emitted.
</dd>
<dt>

<code>preburn_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_PreburnEvent">Libra::PreburnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for preburn requests, and where all
 <code><a href="#0x1_Libra_PreburnEvent">PreburnEvent</a></code>s for this
<code>CoinType</code> will be emitted.
</dd>
<dt>

<code>cancel_burn_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_CancelBurnEvent">Libra::CancelBurnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for all cancelled preburn requests for this
 <code>CoinType</code>.
</dd>
<dt>

<code>exchange_rate_update_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_ToLBRExchangeRateUpdateEvent">Libra::ToLBRExchangeRateUpdateEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for emiting exchange rate change events
</dd>
</dl>


</details>

<a name="0x1_Libra_Preburn"></a>

## Resource `Preburn`

A holding area where funds that will subsequently be burned wait while their underlying
assets are moved off-chain.
This resource can only be created by the holder of a
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a></code>. An account that
contains this address has the authority to initiate a burn request. A burn request can be
resolved by the holder of a
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a></code> by either (1) burning the funds, or (2)
returning the funds to the account that initiated the burn request.
Concurrent preburn requests are not allowed, only one request (in to_burn) can be handled at any time.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>to_burn: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;</code>
</dt>
<dd>
 A single pending burn amount.
 There is no pending burn request if the value in to_burn is 0
</dd>
</dl>


</details>

<a name="0x1_Libra_MAX_SCALING_FACTOR"></a>

## Const `MAX_SCALING_FACTOR`



<pre><code><b>const</b> MAX_SCALING_FACTOR: u64 = 10000000000;
</code></pre>



<a name="0x1_Libra_MAX_U64"></a>

## Const `MAX_U64`

TODO(wrwg): This should be provided somewhere centrally in the framework.


<pre><code><b>const</b> MAX_U64: u64 = 18446744073709551615;
</code></pre>



<a name="0x1_Libra_MAX_U128"></a>

## Const `MAX_U128`



<pre><code><b>const</b> MAX_U128: u128 = 340282366920938463463374607431768211455;
</code></pre>



<a name="0x1_Libra_EBURN_CAPABILITY"></a>

## Const `EBURN_CAPABILITY`

A
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a></code> resource is in an unexpected state.


<pre><code><b>const</b> EBURN_CAPABILITY: u64 = 0;
</code></pre>



<a name="0x1_Libra_ECURRENCY_INFO"></a>

## Const `ECURRENCY_INFO`

A property expected of a
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code> resource didn't hold


<pre><code><b>const</b> ECURRENCY_INFO: u64 = 1;
</code></pre>



<a name="0x1_Libra_EPREBURN"></a>

## Const `EPREBURN`

A property expected of a
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource didn't hold


<pre><code><b>const</b> EPREBURN: u64 = 2;
</code></pre>



<a name="0x1_Libra_EPREBURN_OCCUPIED"></a>

## Const `EPREBURN_OCCUPIED`

The preburn slot is already occupied with coins to be burned.


<pre><code><b>const</b> EPREBURN_OCCUPIED: u64 = 3;
</code></pre>



<a name="0x1_Libra_EPREBURN_EMPTY"></a>

## Const `EPREBURN_EMPTY`

A burn was attempted on
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource that cointained no coins


<pre><code><b>const</b> EPREBURN_EMPTY: u64 = 4;
</code></pre>



<a name="0x1_Libra_EMINTING_NOT_ALLOWED"></a>

## Const `EMINTING_NOT_ALLOWED`

Minting is not allowed for the specified currency


<pre><code><b>const</b> EMINTING_NOT_ALLOWED: u64 = 5;
</code></pre>



<a name="0x1_Libra_EIS_SYNTHETIC_CURRENCY"></a>

## Const `EIS_SYNTHETIC_CURRENCY`

The currency specified is a synthetic (non-fiat) currency


<pre><code><b>const</b> EIS_SYNTHETIC_CURRENCY: u64 = 6;
</code></pre>



<a name="0x1_Libra_ECOIN"></a>

## Const `ECOIN`

A property expected of the coin provided didn't hold


<pre><code><b>const</b> ECOIN: u64 = 7;
</code></pre>



<a name="0x1_Libra_EDESTRUCTION_OF_NONZERO_COIN"></a>

## Const `EDESTRUCTION_OF_NONZERO_COIN`

The destruction of a non-zero coin was attempted. Non-zero coins must be burned.


<pre><code><b>const</b> EDESTRUCTION_OF_NONZERO_COIN: u64 = 8;
</code></pre>



<a name="0x1_Libra_EMINT_CAPABILITY"></a>

## Const `EMINT_CAPABILITY`

A property expected of
<code><a href="#0x1_Libra_MintCapability">MintCapability</a></code> didn't hold


<pre><code><b>const</b> EMINT_CAPABILITY: u64 = 10;
</code></pre>



<a name="0x1_Libra_EAMOUNT_EXCEEDS_COIN_VALUE"></a>

## Const `EAMOUNT_EXCEEDS_COIN_VALUE`

A withdrawal greater than the value of the coin was attempted.


<pre><code><b>const</b> EAMOUNT_EXCEEDS_COIN_VALUE: u64 = 11;
</code></pre>



<a name="0x1_Libra_initialize"></a>

## Function `initialize`

Initialization of the
<code><a href="#0x1_Libra">Libra</a></code> module; initializes the set of
registered currencies in the
<code><a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">0x1::RegisteredCurrencies</a></code> on-chain
config, and publishes the
<code>CurrencyRegistrationCapability</code> under the
<code><a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()</code>. This can only be called from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_initialize">initialize</a>(config_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_initialize">initialize</a>(
    config_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    // Operational constraint
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(config_account);
    <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_initialize">RegisteredCurrencies::initialize</a>(config_account);
}
</code></pre>



</details>

<a name="0x1_Libra_publish_burn_capability"></a>

## Function `publish_burn_capability`

Publishes the
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a></code>
<code>cap</code> for the
<code>CoinType</code> currency under
<code>account</code>.
<code>CoinType</code>
must be a registered currency type. The caller must pass a treasury compliance account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_publish_burn_capability">publish_burn_capability</a>&lt;CoinType&gt;(account: &signer, cap: <a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_publish_burn_capability">publish_burn_capability</a>&lt;CoinType&gt;(
    account: &signer,
    cap: <a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;,
    tc_account: &signer,
) {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <b>assert</b>(
        !exists&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(EBURN_CAPABILITY)
    );
    move_to(account, cap)
}
</code></pre>



</details>

<a name="0x1_Libra_mint"></a>

## Function `mint`

Mints
<code>amount</code> coins. The
<code>account</code> must hold a
<code><a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> at the top-level in order for this call
to be successful, and will fail with
<code>MISSING_DATA</code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_mint">mint</a>&lt;CoinType&gt;(account: &signer, value: u64): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_mint">mint</a>&lt;CoinType&gt;(account: &signer, value: u64): <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x1_Libra_MintCapability">MintCapability</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(EMINT_CAPABILITY));
    <a href="#0x1_Libra_mint_with_capability">mint_with_capability</a>(
        value,
        borrow_global&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(addr)
    )
}
</code></pre>



</details>

<a name="0x1_Libra_burn"></a>

## Function `burn`

Burns the coins currently held in the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource held under
<code>preburn_address</code>.
Calls to this functions will fail if the
<code>account</code> does not have a
published
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a></code> for the
<code>CoinType</code> published under it.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_burn">burn</a>&lt;CoinType&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_burn">burn</a>&lt;CoinType&gt;(
    account: &signer,
    preburn_address: address
) <b>acquires</b> <a href="#0x1_Libra_BurnCapability">BurnCapability</a>, <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x1_Libra_Preburn">Preburn</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(EBURN_CAPABILITY));
    <a href="#0x1_Libra_burn_with_capability">burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(addr)
    )
}
</code></pre>



</details>

<a name="0x1_Libra_cancel_burn"></a>

## Function `cancel_burn`

Cancels the current burn request in the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource held
under the
<code>preburn_address</code>, and returns the coins.
Calls to this will fail if the sender does not have a published
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>, or if there is no preburn request
outstanding in the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource under
<code>preburn_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_cancel_burn">cancel_burn</a>&lt;CoinType&gt;(account: &signer, preburn_address: address): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_cancel_burn">cancel_burn</a>&lt;CoinType&gt;(
    account: &signer,
    preburn_address: address
): <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt; <b>acquires</b> <a href="#0x1_Libra_BurnCapability">BurnCapability</a>, <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x1_Libra_Preburn">Preburn</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(EBURN_CAPABILITY));
    <a href="#0x1_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(addr)
    )
}
</code></pre>



</details>

<a name="0x1_Libra_mint_with_capability"></a>

## Function `mint_with_capability`

Mint a new
<code><a href="#0x1_Libra">Libra</a></code> coin of
<code>CoinType</code> currency worth
<code>value</code>. The
caller must have a reference to a
<code><a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code>. Only
the treasury compliance account or the
<code><a href="LBR.md#0x1_LBR">0x1::LBR</a></code> module can acquire such a
reference.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_mint_with_capability">mint_with_capability</a>&lt;CoinType&gt;(value: u64, _capability: &<a href="#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_mint_with_capability">mint_with_capability</a>&lt;CoinType&gt;(
    value: u64,
    _capability: &<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;
): <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt; <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <b>let</b> currency_code = <a href="#0x1_Libra_currency_code">currency_code</a>&lt;CoinType&gt;();
    // <b>update</b> market cap <b>resource</b> <b>to</b> reflect minting
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    <b>assert</b>(info.can_mint, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(EMINTING_NOT_ALLOWED));
    <b>assert</b>(MAX_U128 - info.total_value &gt;= (value <b>as</b> u128), <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(ECURRENCY_INFO));
    info.total_value = info.total_value + (value <b>as</b> u128);
    // don't emit mint events for synthetic currenices
    <b>if</b> (!info.is_synthetic) {
        <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.mint_events,
            <a href="#0x1_Libra_MintEvent">MintEvent</a>{
                amount: value,
                currency_code,
            }
        );
    };

    <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt; { value }
}
</code></pre>



</details>

<a name="0x1_Libra_preburn_with_resource"></a>

## Function `preburn_with_resource`

Add the
<code>coin</code> to the
<code>preburn</code> to_burn field in the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource
held at the address
<code>preburn_address</code> if it is empty, otherwise raise
a PendingPreburn Error (code 6). Emits a
<code><a href="#0x1_Libra_PreburnEvent">PreburnEvent</a></code> to
the
<code>preburn_events</code> event stream in the
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code> for the
<code>CoinType</code> passed in. However, if the currency being preburned is
<code>synthetic</code> then no
<code><a href="#0x1_Libra_PreburnEvent">PreburnEvent</a></code> event will be emitted.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_preburn_with_resource">preburn_with_resource</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, preburn: &<b>mut</b> <a href="#0x1_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_preburn_with_resource">preburn_with_resource</a>&lt;CoinType&gt;(
    coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;,
    preburn: &<b>mut</b> <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;,
    preburn_address: address,
) <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> coin_value = <a href="#0x1_Libra_value">value</a>(&coin);
    // Throw <b>if</b> already occupied
    <b>assert</b>(<a href="#0x1_Libra_value">value</a>(&preburn.to_burn) == 0, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(EPREBURN_OCCUPIED));
    <a href="#0x1_Libra_deposit">deposit</a>(&<b>mut</b> preburn.to_burn, coin);
    <b>let</b> currency_code = <a href="#0x1_Libra_currency_code">currency_code</a>&lt;CoinType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    <b>assert</b>(MAX_U64 - info.preburn_value &gt;= coin_value, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(ECOIN));
    info.preburn_value = info.preburn_value + coin_value;
    // don't emit preburn events for synthetic currencies
    <b>if</b> (!info.is_synthetic) {
        <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.preburn_events,
            <a href="#0x1_Libra_PreburnEvent">PreburnEvent</a>{
                amount: coin_value,
                currency_code,
                preburn_address,
            }
        );
    };
}
</code></pre>



</details>

<a name="0x1_Libra_create_preburn"></a>

## Function `create_preburn`

Create a
<code><a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code> resource


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_create_preburn">create_preburn</a>&lt;CoinType&gt;(tc_account: &signer): <a href="#0x1_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_create_preburn">create_preburn</a>&lt;CoinType&gt;(
    tc_account: &signer
): <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt; {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt; { to_burn: <a href="#0x1_Libra_zero">zero</a>&lt;CoinType&gt;() }
}
</code></pre>



</details>

<a name="0x1_Libra_publish_preburn_to_account"></a>

## Function `publish_preburn_to_account`

Publishes a
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource under
<code>account</code>. This function is
used for bootstrapping the designated dealer at account-creation
time, and the association TC account
<code>creator</code> (at
<code><a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()</code>) is creating
this resource for the designated dealer.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_publish_preburn_to_account">publish_preburn_to_account</a>&lt;CoinType&gt;(account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_publish_preburn_to_account">publish_preburn_to_account</a>&lt;CoinType&gt;(
    account: &signer,
    tc_account: &signer
) <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="Roles.md#0x1_Roles_assert_designated_dealer">Roles::assert_designated_dealer</a>(account);
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>assert</b>(!<a href="#0x1_Libra_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(EIS_SYNTHETIC_CURRENCY));
    <b>assert</b>(!exists&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(EPREBURN));
    move_to(account, <a href="#0x1_Libra_create_preburn">create_preburn</a>&lt;CoinType&gt;(tc_account))
}
</code></pre>



</details>

<a name="0x1_Libra_preburn_to"></a>

## Function `preburn_to`

Sends
<code>coin</code> to the preburn queue for
<code>account</code>, where it will wait to either be burned
or returned to the balance of
<code>account</code>.
Calls to this function will fail if
<code>account</code> does not have a
<code><a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code> resource published under it.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_preburn_to">preburn_to</a>&lt;CoinType&gt;(account: &signer, coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_preburn_to">preburn_to</a>&lt;CoinType&gt;(
    account: &signer,
    coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;
) <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x1_Libra_Preburn">Preburn</a> {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(sender), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EPREBURN));
    <a href="#0x1_Libra_preburn_with_resource">preburn_with_resource</a>(coin, borrow_global_mut&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(sender), sender);
}
</code></pre>



</details>

<a name="0x1_Libra_burn_with_capability"></a>

## Function `burn_with_capability`

Permanently removes the coins held in the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource (in to_burn field)
stored at
<code>preburn_address</code> and updates the market cap accordingly.
This function can only be called by the holder of a
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>.
Calls to this function will fail if the there is no
<code><a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code>
resource under
<code>preburn_address</code>, or, if the preburn to_burn area for
<code>CoinType</code> is empty.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_burn_with_capability">burn_with_capability</a>&lt;CoinType&gt;(preburn_address: address, capability: &<a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_burn_with_capability">burn_with_capability</a>&lt;CoinType&gt;(
    preburn_address: address,
    capability: &<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
) <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x1_Libra_Preburn">Preburn</a> {
    // destroy the coin in the preburn to_burn area
    <b>assert</b>(exists&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EPREBURN));
    <a href="#0x1_Libra_burn_with_resource_cap">burn_with_resource_cap</a>(
        borrow_global_mut&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address),
        preburn_address,
        capability
    )
}
</code></pre>



</details>

<a name="0x1_Libra_burn_with_resource_cap"></a>

## Function `burn_with_resource_cap`

Permanently removes the coins held in the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource (in to_burn field)
stored at
<code>preburn_address</code> and updates the market cap accordingly.
This function can only be called by the holder of a
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>.
Calls to this function will fail if the there is no
<code><a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code>
resource under
<code>preburn_address</code>, or, if the preburn to_burn area for
<code>CoinType</code> is empty (error code 7).


<pre><code><b>fun</b> <a href="#0x1_Libra_burn_with_resource_cap">burn_with_resource_cap</a>&lt;CoinType&gt;(preburn: &<b>mut</b> <a href="#0x1_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;, preburn_address: address, _capability: &<a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Libra_burn_with_resource_cap">burn_with_resource_cap</a>&lt;CoinType&gt;(
    preburn: &<b>mut</b> <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;,
    preburn_address: address,
    _capability: &<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
) <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> currency_code = <a href="#0x1_Libra_currency_code">currency_code</a>&lt;CoinType&gt;();
    // Abort <b>if</b> no coin present in preburn area
    <b>assert</b>(preburn.to_burn.value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(EPREBURN_EMPTY));
    // destroy the coin in <a href="#0x1_Libra_Preburn">Preburn</a> area
    <b>let</b> <a href="#0x1_Libra">Libra</a> { value } = <a href="#0x1_Libra_withdraw_all">withdraw_all</a>&lt;CoinType&gt;(&<b>mut</b> preburn.to_burn);
    // <b>update</b> the market cap
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    <b>assert</b>(info.total_value &gt;= (value <b>as</b> u128), <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(ECURRENCY_INFO));
    info.total_value = info.total_value - (value <b>as</b> u128);
    <b>assert</b>(info.preburn_value &gt;= value, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(EPREBURN));
    info.preburn_value = info.preburn_value - value;
    // don't emit burn events for synthetic currencies
    <b>if</b> (!info.is_synthetic) {
        <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.burn_events,
            <a href="#0x1_Libra_BurnEvent">BurnEvent</a> {
                amount: value,
                currency_code,
                preburn_address,
            }
        );
    };
}
</code></pre>



</details>

<a name="0x1_Libra_cancel_burn_with_capability"></a>

## Function `cancel_burn_with_capability`

Cancels the burn request in the
<code><a href="#0x1_Libra_Preburn">Preburn</a></code> resource stored at
<code>preburn_address</code> and
return the coins to the caller.
This function can only be called by the holder of a
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>, and will fail if the
<code><a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code> resource
at
<code>preburn_address</code> does not contain a pending burn request.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;CoinType&gt;(preburn_address: address, _capability: &<a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;CoinType&gt;(
    preburn_address: address,
    _capability: &<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
): <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt; <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x1_Libra_Preburn">Preburn</a> {
    // destroy the coin in the preburn area
    <b>assert</b>(exists&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EPREBURN));
    <b>let</b> preburn = borrow_global_mut&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address);
    <b>let</b> coin = <a href="#0x1_Libra_withdraw_all">withdraw_all</a>&lt;CoinType&gt;(&<b>mut</b> preburn.to_burn);
    // <b>update</b> the market cap
    <b>let</b> currency_code = <a href="#0x1_Libra_currency_code">currency_code</a>&lt;CoinType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    <b>let</b> amount = <a href="#0x1_Libra_value">value</a>(&coin);
    <b>assert</b>(info.preburn_value &gt;= amount, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(EPREBURN));
    info.preburn_value = info.preburn_value - amount;
    // Don't emit cancel burn events for synthetic currencies. cancel burn shouldn't be be used
    // for synthetics in the first place
    <b>if</b> (!info.is_synthetic) {
        <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.cancel_burn_events,
            <a href="#0x1_Libra_CancelBurnEvent">CancelBurnEvent</a> {
                amount,
                currency_code,
                preburn_address,
            }
        );
    };

    coin
}
</code></pre>



</details>

<a name="0x1_Libra_burn_now"></a>

## Function `burn_now`

A shortcut for immediately burning a coin. This calls preburn followed by a subsequent burn, and is
used for administrative burns, like unpacking an LBR coin or charging fees.
> TODO(wrwg): consider removing complexity here by removing the need for a preburn resource. The preburn
> resource is required to have 0 value on entry and will have so also after this call, so it is redundant.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_burn_now">burn_now</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, preburn: &<b>mut</b> <a href="#0x1_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;, preburn_address: address, capability: &<a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_burn_now">burn_now</a>&lt;CoinType&gt;(
    coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;,
    preburn: &<b>mut</b> <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;,
    preburn_address: address,
    capability: &<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
) <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>assert</b>(coin.value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(ECOIN));
    <a href="#0x1_Libra_preburn_with_resource">preburn_with_resource</a>(coin, preburn, preburn_address);
    <a href="#0x1_Libra_burn_with_resource_cap">burn_with_resource_cap</a>(preburn, preburn_address, capability);
}
</code></pre>



</details>

<a name="0x1_Libra_remove_burn_capability"></a>

## Function `remove_burn_capability`

Removes and returns the
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> from
<code>account</code>.
Calls to this function will fail if
<code>account</code> does  not have a
published
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> resource at the top-level.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_remove_burn_capability">remove_burn_capability</a>&lt;CoinType&gt;(account: &signer): <a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_remove_burn_capability">remove_burn_capability</a>&lt;CoinType&gt;(account: &signer): <a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
<b>acquires</b> <a href="#0x1_Libra_BurnCapability">BurnCapability</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(EBURN_CAPABILITY));
    move_from&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_Libra_preburn_value"></a>

## Function `preburn_value`

Returns the total value of
<code><a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;</code> that is waiting to be
burned throughout the system (i.e. the sum of all outstanding
preburn requests across all preburn resources for the
<code>CoinType</code>
currency).


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_preburn_value">preburn_value</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_preburn_value">preburn_value</a>&lt;CoinType&gt;(): u64 <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    borrow_global&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).preburn_value
}
</code></pre>



</details>

<a name="0x1_Libra_zero"></a>

## Function `zero`

Create a new
<code><a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;</code> with a value of
<code>0</code>. Anyone can call
this and it will be successful as long as
<code>CoinType</code> is a registered currency.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_zero">zero</a>&lt;CoinType&gt;(): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_zero">zero</a>&lt;CoinType&gt;(): <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt; {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt; { value: 0 }
}
</code></pre>



</details>

<a name="0x1_Libra_value"></a>

## Function `value`

Returns the
<code>value</code> of the passed in
<code>coin</code>. The value is
represented in the base units for the currency represented by
<code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_value">value</a>&lt;CoinType&gt;(coin: &<a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_value">value</a>&lt;CoinType&gt;(coin: &<a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;): u64 {
    coin.value
}
</code></pre>



</details>

<a name="0x1_Libra_split"></a>

## Function `split`

Removes
<code>amount</code> of value from the passed in
<code>coin</code>. Returns the
remaining balance of the passed in
<code>coin</code>, along with another coin
with value equal to
<code>amount</code>. Calls will fail if
<code>amount &gt; <a href="#0x1_Libra_value">Libra::value</a>(&coin)</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_split">split</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, amount: u64): (<a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_split">split</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;, amount: u64): (<a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;, <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;) {
    <b>let</b> other = <a href="#0x1_Libra_withdraw">withdraw</a>(&<b>mut</b> coin, amount);
    (coin, other)
}
</code></pre>



</details>

<a name="0x1_Libra_withdraw"></a>

## Function `withdraw`

Withdraw
<code>amount</code> from the passed-in
<code>coin</code>, where the original coin is modified in place.
After this function is executed, the original
<code>coin</code> will have
<code>value = original_value - amount</code>, and the new coin will have a
<code>value = amount</code>.
Calls will abort if the passed-in
<code>amount</code> is greater than the
value of the passed-in
<code>coin</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_withdraw">withdraw</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, amount: u64): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_withdraw">withdraw</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;, amount: u64): <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt; {
    // Check that `amount` is less than the coin's value
    <b>assert</b>(coin.value &gt;= amount, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(EAMOUNT_EXCEEDS_COIN_VALUE));
    coin.value = coin.value - amount;
    <a href="#0x1_Libra">Libra</a> { value: amount }
}
</code></pre>



</details>

<a name="0x1_Libra_withdraw_all"></a>

## Function `withdraw_all`

Return a
<code><a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;</code> worth
<code>coin.value</code> and reduces the
<code>value</code> of the input
<code>coin</code> to
zero. Does not abort.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_withdraw_all">withdraw_all</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_withdraw_all">withdraw_all</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;): <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt; {
    <b>let</b> val = coin.value;
    <a href="#0x1_Libra_withdraw">withdraw</a>(coin, val)
}
</code></pre>



</details>

<a name="0x1_Libra_join"></a>

## Function `join`

and returns a new coin whose value is equal to the sum of the two inputs.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_join">join</a>&lt;CoinType&gt;(coin1: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, coin2: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_join">join</a>&lt;CoinType&gt;(coin1: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;, coin2: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;): <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;  {
    <a href="#0x1_Libra_deposit">deposit</a>(&<b>mut</b> coin1, coin2);
    coin1
}
</code></pre>



</details>

<a name="0x1_Libra_deposit"></a>

## Function `deposit`

"Merges" the two coins.
The coin passed in by reference will have a value equal to the sum of the two coins
The
<code>check</code> coin is consumed in the process


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_deposit">deposit</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, check: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_deposit">deposit</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;, check: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="#0x1_Libra">Libra</a> { value } = check;
    <b>assert</b>(MAX_U64 - coin.value &gt;= value, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(ECOIN));
    coin.value = coin.value + value;
}
</code></pre>



</details>

<a name="0x1_Libra_destroy_zero"></a>

## Function `destroy_zero`

Destroy a zero-value coin. Calls will fail if the
<code>value</code> in the passed-in
<code>coin</code> is non-zero
so you cannot "burn" any non-zero amount of
<code><a href="#0x1_Libra">Libra</a></code> without having
a
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a></code> for the specific
<code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="#0x1_Libra">Libra</a> { value } = coin;
    <b>assert</b>(value == 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(EDESTRUCTION_OF_NONZERO_COIN))
}
</code></pre>



</details>

<a name="0x1_Libra_register_currency"></a>

## Function `register_currency`

Register the type
<code>CoinType</code> as a currency. Until the type is
registered as a currency it cannot be used as a coin/currency unit in Libra.
The passed-in
<code>lr_account</code> must be a specific address (
<code><a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code>) and
<code>lr_account</code> must also have the correct
<code><a href="#0x1_Libra_RegisterNewCurrency">RegisterNewCurrency</a></code> capability.
After the first registration of
<code>CoinType</code> as a
currency, additional attempts to register
<code>CoinType</code> as a currency
will abort.
When the
<code>CoinType</code> is registered it publishes the
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;</code> resource under the
<code><a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code> and
adds the currency to the set of
<code><a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a></code>. It returns
<code><a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> and
<code><a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> resources.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(lr_account: &signer, to_lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>, is_synthetic: bool, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;): (<a href="#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;, <a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(
    lr_account: &signer,
    to_lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: vector&lt;u8&gt;,
): (<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;, <a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;)
{
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    // Operational constraint that it must be stored under a specific address.
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_currency_info">CoreAddresses::assert_currency_info</a>(lr_account);
    <b>assert</b>(
        !exists&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(ECURRENCY_INFO)
    );
    <b>assert</b>(0 &lt; scaling_factor && <a href="#0x1_Libra_scaling_factor">scaling_factor</a> &lt;= MAX_SCALING_FACTOR, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(ECURRENCY_INFO));
    move_to(lr_account, <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt; {
        total_value: 0,
        preburn_value: 0,
        to_lbr_exchange_rate,
        is_synthetic,
        scaling_factor,
        fractional_part,
        currency_code: <b>copy</b> currency_code,
        can_mint: <b>true</b>,
        mint_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Libra_MintEvent">MintEvent</a>&gt;(lr_account),
        burn_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Libra_BurnEvent">BurnEvent</a>&gt;(lr_account),
        preburn_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Libra_PreburnEvent">PreburnEvent</a>&gt;(lr_account),
        cancel_burn_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Libra_CancelBurnEvent">CancelBurnEvent</a>&gt;(lr_account),
        exchange_rate_update_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Libra_ToLBRExchangeRateUpdateEvent">ToLBRExchangeRateUpdateEvent</a>&gt;(lr_account)
    });
    <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_add_currency_code">RegisteredCurrencies::add_currency_code</a>(
        lr_account,
        currency_code,
    );
    (<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;{}, <a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;{})
}
</code></pre>



</details>

<a name="0x1_Libra_register_SCS_currency"></a>

## Function `register_SCS_currency`

Registers a stable currency (SCS) coin -- i.e., a non-synthetic currency.
Resources are published on two distinct
accounts: The CoinInfo is published on the Libra root account, and the mint and
burn capabilities are published on a treasury compliance account.
This code allows different currencies to have different treasury compliance
accounts.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_register_SCS_currency">register_SCS_currency</a>&lt;CoinType&gt;(lr_account: &signer, tc_account: &signer, to_lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_register_SCS_currency">register_SCS_currency</a>&lt;CoinType&gt;(
    lr_account: &signer,
    tc_account: &signer,
    to_lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: vector&lt;u8&gt;,
) {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>let</b> (mint_cap, burn_cap) =
        <a href="#0x1_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(
            lr_account,
            to_lbr_exchange_rate,
            <b>false</b>,   // is_synthetic
            scaling_factor,
            fractional_part,
            currency_code,
        );
    <b>assert</b>(
        !exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(tc_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(EMINT_CAPABILITY)
    );
    move_to(tc_account, mint_cap);
    <a href="#0x1_Libra_publish_burn_capability">publish_burn_capability</a>&lt;CoinType&gt;(tc_account, burn_cap, tc_account);
}
</code></pre>



</details>

<a name="0x1_Libra_market_cap"></a>

## Function `market_cap`

Returns the total amount of currency minted of type
<code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_market_cap">market_cap</a>&lt;CoinType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_market_cap">market_cap</a>&lt;CoinType&gt;(): u128
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    borrow_global&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).total_value
}
</code></pre>



</details>

<a name="0x1_Libra_approx_lbr_for_value"></a>

## Function `approx_lbr_for_value`

Returns the value of the coin in the
<code>FromCoinType</code> currency in LBR.
This should only be used where a _rough_ approximation of the exchange
rate is needed.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value: u64): u64
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> lbr_exchange_rate = <a href="#0x1_Libra_lbr_exchange_rate">lbr_exchange_rate</a>&lt;FromCoinType&gt;();
    <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(from_value, lbr_exchange_rate)
}
</code></pre>



</details>

<a name="0x1_Libra_approx_lbr_for_coin"></a>

## Function `approx_lbr_for_coin`

Returns the value of the coin in the
<code>FromCoinType</code> currency in LBR.
This should only be used where a rough approximation of the exchange
rate is needed.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_approx_lbr_for_coin">approx_lbr_for_coin</a>&lt;FromCoinType&gt;(coin: &<a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;FromCoinType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_approx_lbr_for_coin">approx_lbr_for_coin</a>&lt;FromCoinType&gt;(coin: &<a href="#0x1_Libra">Libra</a>&lt;FromCoinType&gt;): u64
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> from_value = <a href="#0x1_Libra_value">value</a>(coin);
    <a href="#0x1_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value)
}
</code></pre>



</details>

<a name="0x1_Libra_is_currency"></a>

## Function `is_currency`

Returns
<code><b>true</b></code> if the type
<code>CoinType</code> is a registered currency.
Returns
<code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(): bool {
    exists&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_Libra_is_SCS_currency"></a>

## Function `is_SCS_currency`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_is_SCS_currency">is_SCS_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_is_SCS_currency">is_SCS_currency</a>&lt;CoinType&gt;(): bool <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <b>let</b> info = borrow_global&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    !info.is_synthetic
}
</code></pre>



</details>

<a name="0x1_Libra_is_synthetic_currency"></a>

## Function `is_synthetic_currency`

Returns
<code><b>true</b></code> if
<code>CoinType</code> is a synthetic currency as defined in
its
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code>. Returns
<code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(): bool
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>();
    exists&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(addr) &&
        borrow_global&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(addr).is_synthetic
}
</code></pre>



</details>

<a name="0x1_Libra_scaling_factor"></a>

## Function `scaling_factor`

Returns the scaling factor for the
<code>CoinType</code> currency as defined
in its
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_scaling_factor">scaling_factor</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_scaling_factor">scaling_factor</a>&lt;CoinType&gt;(): u64
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    borrow_global&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).scaling_factor
}
</code></pre>



</details>

<a name="0x1_Libra_fractional_part"></a>

## Function `fractional_part`

Returns the representable (i.e. real-world) fractional part for the
<code>CoinType</code> currency as defined in its
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_fractional_part">fractional_part</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_fractional_part">fractional_part</a>&lt;CoinType&gt;(): u64
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    borrow_global&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).fractional_part
}
</code></pre>



</details>

<a name="0x1_Libra_currency_code"></a>

## Function `currency_code`

Returns the currency code for the registered currency as defined in
its
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_currency_code">currency_code</a>&lt;CoinType&gt;(): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_currency_code">currency_code</a>&lt;CoinType&gt;(): vector&lt;u8&gt;
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    *&borrow_global&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).currency_code
}
</code></pre>



</details>

<a name="0x1_Libra_update_lbr_exchange_rate"></a>

## Function `update_lbr_exchange_rate`

Updates the
<code>to_lbr_exchange_rate</code> held in the
<code><a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a></code> for
<code>FromCoinType</code> to the new passed-in
<code>lbr_exchange_rate</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_update_lbr_exchange_rate">update_lbr_exchange_rate</a>&lt;FromCoinType&gt;(tc_account: &signer, lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_update_lbr_exchange_rate">update_lbr_exchange_rate</a>&lt;FromCoinType&gt;(
    tc_account: &signer,
    lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>
) <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;FromCoinType&gt;();
    <b>let</b> currency_info = borrow_global_mut&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;FromCoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    currency_info.to_lbr_exchange_rate = lbr_exchange_rate;
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> currency_info.exchange_rate_update_events,
        <a href="#0x1_Libra_ToLBRExchangeRateUpdateEvent">ToLBRExchangeRateUpdateEvent</a> {
            currency_code: *&currency_info.currency_code,
            new_to_lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32_get_raw_value">FixedPoint32::get_raw_value</a>(*&currency_info.to_lbr_exchange_rate),
        }
    );
}
</code></pre>



</details>

<a name="0x1_Libra_lbr_exchange_rate"></a>

## Function `lbr_exchange_rate`

Returns the (rough) exchange rate between
<code>CoinType</code> and
<code><a href="LBR.md#0x1_LBR">LBR</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_lbr_exchange_rate">lbr_exchange_rate</a>&lt;CoinType&gt;(): <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_lbr_exchange_rate">lbr_exchange_rate</a>&lt;CoinType&gt;(): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    *&borrow_global&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).to_lbr_exchange_rate
}
</code></pre>



</details>

<a name="0x1_Libra_update_minting_ability"></a>

## Function `update_minting_ability`

There may be situations in which we disallow the further minting of
coins in the system without removing the currency. This function
allows the association TC account to control whether or not further coins of
<code>CoinType</code> can be minted or not. If this is called with
<code>can_mint =
<b>true</b></code>, then minting is allowed, if
<code>can_mint = <b>false</b></code> then minting is
disallowed until it is turned back on via this function. All coins
start out in the default state of
<code>can_mint = <b>true</b></code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_update_minting_ability">update_minting_ability</a>&lt;CoinType&gt;(tc_account: &signer, can_mint: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_update_minting_ability">update_minting_ability</a>&lt;CoinType&gt;(
    tc_account: &signer,
    can_mint: bool,
    )
<b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <b>let</b> currency_info = borrow_global_mut&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    currency_info.can_mint = can_mint;
}
</code></pre>



</details>

<a name="0x1_Libra_assert_is_currency"></a>

## Function `assert_is_currency`

Asserts that
<code>CoinType</code> is a registered currency.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;() {
    <b>assert</b>(<a href="#0x1_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(ECURRENCY_INFO));
}
</code></pre>



</details>

<a name="0x1_Libra_assert_is_SCS_currency"></a>

## Function `assert_is_SCS_currency`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_assert_is_SCS_currency">assert_is_SCS_currency</a>&lt;CoinType&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_assert_is_SCS_currency">assert_is_SCS_currency</a>&lt;CoinType&gt;() <b>acquires</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <b>assert</b>(<a href="#0x1_Libra_is_SCS_currency">is_SCS_currency</a>&lt;CoinType&gt;(), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(ECURRENCY_INFO));
}
</code></pre>



</details>

<a name="0x1_Libra_Specification"></a>

## Specification


<a name="0x1_Libra_Specification_CurrencyInfo"></a>

### Resource `CurrencyInfo`


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;
</code></pre>



<dl>
<dt>

<code>total_value: u128</code>
</dt>
<dd>
 The total value for the currency represented by
<code>CoinType</code>. Mutable.
</dd>
<dt>

<code>preburn_value: u64</code>
</dt>
<dd>
 Value of funds that are in the process of being burned.  Mutable.
</dd>
<dt>

<code>to_lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a></code>
</dt>
<dd>
 The (rough) exchange rate from
<code>CoinType</code> to
<code><a href="LBR.md#0x1_LBR">LBR</a></code>. Mutable.
</dd>
<dt>

<code>is_synthetic: bool</code>
</dt>
<dd>
 Holds whether or not this currency is synthetic (contributes to the
 off-chain reserve) or not. An example of such a synthetic
currency would be the LBR.
</dd>
<dt>

<code>scaling_factor: u64</code>
</dt>
<dd>
 The scaling factor for the coin (i.e. the amount to multiply by
 to get to the human-readable representation for this currency).
 e.g. 10^6 for
<code><a href="Coin1.md#0x1_Coin1">Coin1</a></code>

 > TODO(wrwg): should the above be "to divide by"?
</dd>
<dt>

<code>fractional_part: u64</code>
</dt>
<dd>
 The smallest fractional part (number of decimal places) to be
 used in the human-readable representation for the currency (e.g.
 10^2 for
<code><a href="Coin1.md#0x1_Coin1">Coin1</a></code> cents)
</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 The code symbol for this
<code>CoinType</code>. ASCII encoded.
 e.g. for "LBR" this is x"4C4252". No character limit.
</dd>
<dt>

<code>can_mint: bool</code>
</dt>
<dd>
 We may want to disable the ability to mint further coins of a
 currency while that currency is still around. This allows us to
 keep the currency in circulation while disallowing further
 creation of coins in the
<code>CoinType</code> currency. Mutable.
</dd>
<dt>

<code>mint_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_MintEvent">Libra::MintEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for minting and where
<code><a href="#0x1_Libra_MintEvent">MintEvent</a></code>s will be emitted.
</dd>
<dt>

<code>burn_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_BurnEvent">Libra::BurnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for burning, and where
<code><a href="#0x1_Libra_BurnEvent">BurnEvent</a></code>s will be emitted.
</dd>
<dt>

<code>preburn_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_PreburnEvent">Libra::PreburnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for preburn requests, and where all
 <code><a href="#0x1_Libra_PreburnEvent">PreburnEvent</a></code>s for this
<code>CoinType</code> will be emitted.
</dd>
<dt>

<code>cancel_burn_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_CancelBurnEvent">Libra::CancelBurnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for all cancelled preburn requests for this
 <code>CoinType</code>.
</dd>
<dt>

<code>exchange_rate_update_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Libra_ToLBRExchangeRateUpdateEvent">Libra::ToLBRExchangeRateUpdateEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for emiting exchange rate change events
</dd>
</dl>



<pre><code><b>invariant</b> 0 &lt; scaling_factor && <a href="#0x1_Libra_scaling_factor">scaling_factor</a> &lt;= MAX_SCALING_FACTOR;
</code></pre>



<a name="0x1_Libra_Specification_publish_burn_capability"></a>

### Function `publish_burn_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_publish_burn_capability">publish_burn_capability</a>&lt;CoinType&gt;(account: &signer, cap: <a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;, tc_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;();
<b>include</b> <a href="#0x1_Libra_PublishBurnCapAbortsIfs">PublishBurnCapAbortsIfs</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_Libra_PublishBurnCapAbortsIfs"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_PublishBurnCapAbortsIfs">PublishBurnCapAbortsIfs</a>&lt;CoinType&gt; {
    account: &signer;
    tc_account: &signer;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
    <b>aborts_if</b> exists&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) with Errors::ALREADY_PUBLISHED;
}
</code></pre>


Returns true if a BurnCapability for CoinType exists at addr.


<a name="0x1_Libra_spec_has_burn_cap"></a>


<pre><code><b>define</b> <a href="#0x1_Libra_spec_has_burn_cap">spec_has_burn_cap</a>&lt;CoinType&gt;(addr: address): bool {
exists&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(addr)
}
</code></pre>



<a name="0x1_Libra_Specification_mint"></a>

### Function `mint`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_mint">mint</a>&lt;CoinType&gt;(account: &signer, value: u64): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



Must abort if the account does not have the MintCapability [B11].


<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) with Errors::REQUIRES_CAPABILITY;
<b>include</b> <a href="#0x1_Libra_MintAbortsIf">MintAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="#0x1_Libra_MintEnsures">MintEnsures</a>&lt;CoinType&gt;;
</code></pre>



<a name="0x1_Libra_Specification_burn"></a>

### Function `burn`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_burn">burn</a>&lt;CoinType&gt;(account: &signer, preburn_address: address)
</code></pre>



Must abort if the account does not have the BurnCapability [B12].


<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) with Errors::REQUIRES_CAPABILITY;
<b>aborts_if</b> !exists&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address) with Errors::NOT_PUBLISHED;
<b>include</b> <a href="#0x1_Libra_BurnAbortsIf">BurnAbortsIf</a>&lt;CoinType&gt;{preburn: <b>global</b>&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address)};
</code></pre>



<a name="0x1_Libra_Specification_cancel_burn"></a>

### Function `cancel_burn`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_cancel_burn">cancel_burn</a>&lt;CoinType&gt;(account: &signer, preburn_address: address): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>




<pre><code>pragma aborts_if_is_partial = <b>true</b>;
</code></pre>


Must abort if the account does not have the BurnCapability [B12].


<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) with Errors::REQUIRES_CAPABILITY;
aborts_with Errors::NOT_PUBLISHED, Errors::LIMIT_EXCEEDED;
</code></pre>



<a name="0x1_Libra_Specification_mint_with_capability"></a>

### Function `mint_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_mint_with_capability">mint_with_capability</a>&lt;CoinType&gt;(value: u64, _capability: &<a href="#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>




<pre><code>pragma opaque;
<b>modifies</b> <b>global</b>&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
<a name="0x1_Libra_currency_info$54"></a>
<b>let</b> currency_info = <b>global</b>&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
<b>include</b> <a href="#0x1_Libra_MintAbortsIf">MintAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="#0x1_Libra_MintEnsures">MintEnsures</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_Libra_MintAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_MintAbortsIf">MintAbortsIf</a>&lt;CoinType&gt; {
    value: u64;
    <b>include</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;CoinType&gt;;
    <b>aborts_if</b> !<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().can_mint with Errors::INVALID_STATE;
    <b>aborts_if</b> <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value + value &gt; max_u128() with Errors::LIMIT_EXCEEDED;
}
</code></pre>




<a name="0x1_Libra_MintEnsures"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_MintEnsures">MintEnsures</a>&lt;CoinType&gt; {
    value: u64;
    result: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;;
    <a name="0x1_Libra_currency_info$49"></a>
    <b>let</b> currency_info = <b>global</b>&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    <b>ensures</b> exists&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    <b>ensures</b> currency_info
        == update_field(<b>old</b>(currency_info), total_value, <b>old</b>(currency_info.total_value) + value);
    <b>ensures</b> result.value == value;
}
</code></pre>



<a name="0x1_Libra_Specification_preburn_with_resource"></a>

### Function `preburn_with_resource`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_preburn_with_resource">preburn_with_resource</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, preburn: &<b>mut</b> <a href="#0x1_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;, preburn_address: address)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Libra_PreburnWithResourceAbortsIf">PreburnWithResourceAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="#0x1_Libra_PreburnEnsures">PreburnEnsures</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_Libra_PreburnWithResourceAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_PreburnWithResourceAbortsIf">PreburnWithResourceAbortsIf</a>&lt;CoinType&gt; {
    coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;;
    preburn: <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;;
    <b>aborts_if</b> preburn.to_burn.value != 0 with Errors::INVALID_STATE;
    <b>include</b> <a href="#0x1_Libra_PreburnAbortsIf">PreburnAbortsIf</a>&lt;CoinType&gt;;
}
</code></pre>




<a name="0x1_Libra_PreburnAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_PreburnAbortsIf">PreburnAbortsIf</a>&lt;CoinType&gt; {
    coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;;
    <b>include</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;CoinType&gt;;
    <b>aborts_if</b> <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value + coin.value &gt; MAX_U64 with Errors::LIMIT_EXCEEDED;
}
</code></pre>




<a name="0x1_Libra_PreburnEnsures"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_PreburnEnsures">PreburnEnsures</a>&lt;CoinType&gt; {
    coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;;
    preburn: <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;;
    <b>ensures</b> <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value
                == <b>old</b>(<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value) + coin.value;
}
</code></pre>



<a name="0x1_Libra_Specification_publish_preburn_to_account"></a>

### Function `publish_preburn_to_account`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_publish_preburn_to_account">publish_preburn_to_account</a>&lt;CoinType&gt;(account: &signer, tc_account: &signer)
</code></pre>




<pre><code><b>modifies</b> <b>global</b>&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDesignatedDealer">Roles::AbortsIfNotDesignatedDealer</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;CoinType&gt;;
<b>aborts_if</b> <a href="#0x1_Libra_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;() with Errors::INVALID_ARGUMENT;
<b>aborts_if</b> exists&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) with Errors::ALREADY_PUBLISHED;
</code></pre>



<a name="0x1_Libra_Specification_preburn_to"></a>

### Function `preburn_to`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_preburn_to">preburn_to</a>&lt;CoinType&gt;(account: &signer, coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



Must abort if the account does have the Preburn [B13].


<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) with Errors::NOT_PUBLISHED;
<b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).to_burn.value != 0
    with Errors::INVALID_STATE;
<b>include</b> <a href="#0x1_Libra_PreburnAbortsIf">PreburnAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="#0x1_Libra_PreburnEnsures">PreburnEnsures</a>&lt;CoinType&gt;{preburn: <b>global</b>&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account))};
</code></pre>



<a name="0x1_Libra_Specification_burn_with_capability"></a>

### Function `burn_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_burn_with_capability">burn_with_capability</a>&lt;CoinType&gt;(preburn_address: address, capability: &<a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address) with Errors::NOT_PUBLISHED;
<b>include</b> <a href="#0x1_Libra_BurnAbortsIf">BurnAbortsIf</a>&lt;CoinType&gt;{preburn: <b>global</b>&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address)};
<b>include</b> <a href="#0x1_Libra_BurnEnsures">BurnEnsures</a>&lt;CoinType&gt;{preburn: <b>global</b>&lt;<a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address)};
</code></pre>



<a name="0x1_Libra_Specification_burn_with_resource_cap"></a>

### Function `burn_with_resource_cap`


<pre><code><b>fun</b> <a href="#0x1_Libra_burn_with_resource_cap">burn_with_resource_cap</a>&lt;CoinType&gt;(preburn: &<b>mut</b> <a href="#0x1_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;, preburn_address: address, _capability: &<a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Libra_BurnAbortsIf">BurnAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="#0x1_Libra_BurnEnsures">BurnEnsures</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_Libra_BurnAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_BurnAbortsIf">BurnAbortsIf</a>&lt;CoinType&gt; {
    preburn: <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;;
    <b>include</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;CoinType&gt;;
    <a name="0x1_Libra_to_burn$50"></a>
    <b>let</b> to_burn = preburn.to_burn.value;
    <a name="0x1_Libra_info$51"></a>
    <b>let</b> info = <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;();
    <b>aborts_if</b> to_burn == 0 with Errors::INVALID_STATE;
    <b>aborts_if</b> info.total_value &lt; to_burn with Errors::LIMIT_EXCEEDED;
    <b>aborts_if</b> info.<a href="#0x1_Libra_preburn_value">preburn_value</a> &lt; to_burn with Errors::LIMIT_EXCEEDED;
}
</code></pre>




<a name="0x1_Libra_BurnEnsures"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_BurnEnsures">BurnEnsures</a>&lt;CoinType&gt; {
    preburn: <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;;
    <b>ensures</b> <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value
            == <b>old</b>(<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value) - <b>old</b>(preburn.to_burn.value);
    <b>ensures</b> <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value
            == <b>old</b>(<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value) - <b>old</b>(preburn.to_burn.value);
}
</code></pre>



<a name="0x1_Libra_Specification_burn_now"></a>

### Function `burn_now`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_burn_now">burn_now</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, preburn: &<b>mut</b> <a href="#0x1_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;, preburn_address: address, capability: &<a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Libra_BurnNowAbortsIf">BurnNowAbortsIf</a>&lt;CoinType&gt;;
<b>ensures</b> preburn.to_burn.value == 0;
<a name="0x1_Libra_info$55"></a>
<b>let</b> info = <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;();
<b>ensures</b> info.total_value == <b>old</b>(info.total_value) - coin.value;
</code></pre>




<a name="0x1_Libra_BurnNowAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_BurnNowAbortsIf">BurnNowAbortsIf</a>&lt;CoinType&gt; {
    coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;;
    preburn: <a href="#0x1_Libra_Preburn">Preburn</a>&lt;CoinType&gt;;
    <b>aborts_if</b> coin.value == 0 with Errors::INVALID_ARGUMENT;
    <b>include</b> <a href="#0x1_Libra_PreburnWithResourceAbortsIf">PreburnWithResourceAbortsIf</a>&lt;CoinType&gt;;
    <a name="0x1_Libra_info$52"></a>
    <b>let</b> info = <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;();
    <b>aborts_if</b> info.total_value &lt; coin.value with Errors::LIMIT_EXCEEDED;
}
</code></pre>



<a name="0x1_Libra_Specification_remove_burn_capability"></a>

### Function `remove_burn_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_remove_burn_capability">remove_burn_capability</a>&lt;CoinType&gt;(account: &signer): <a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Libra_AbortsIfNoBurnCapability">AbortsIfNoBurnCapability</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_Libra_AbortsIfNoBurnCapability"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_AbortsIfNoBurnCapability">AbortsIfNoBurnCapability</a>&lt;CoinType&gt; {
    account: signer;
    <b>aborts_if</b> !exists&lt;<a href="#0x1_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) with Errors::REQUIRES_CAPABILITY;
}
</code></pre>



<a name="0x1_Libra_Specification_split"></a>

### Function `split`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_split">split</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, amount: u64): (<a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> coin.<a href="#0x1_Libra_value">value</a> &lt; amount with Errors::LIMIT_EXCEEDED;
<b>ensures</b> result_1.value == coin.value - amount;
<b>ensures</b> result_2.value == amount;
</code></pre>



<a name="0x1_Libra_Specification_withdraw"></a>

### Function `withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_withdraw">withdraw</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, amount: u64): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Libra_WithdrawAbortsIf">WithdrawAbortsIf</a>&lt;CoinType&gt;;
<b>ensures</b> coin.value == <b>old</b>(coin.value) - amount;
<b>ensures</b> result.value == amount;
</code></pre>




<a name="0x1_Libra_WithdrawAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_WithdrawAbortsIf">WithdrawAbortsIf</a>&lt;CoinType&gt; {
    coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;;
    amount: u64;
    <b>aborts_if</b> coin.<a href="#0x1_Libra_value">value</a> &lt; amount with Errors::LIMIT_EXCEEDED;
}
</code></pre>



<a name="0x1_Libra_Specification_withdraw_all"></a>

### Function `withdraw_all`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_withdraw_all">withdraw_all</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>




<pre><code>pragma opaque;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result.value == <b>old</b>(coin.value);
<b>ensures</b> coin.value == 0;
</code></pre>



<a name="0x1_Libra_Specification_join"></a>

### Function `join`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_join">join</a>&lt;CoinType&gt;(coin1: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, coin2: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;): <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>




<pre><code>pragma opaque;
<b>aborts_if</b> coin1.value + coin2.value &gt; max_u64() with Errors::LIMIT_EXCEEDED;
<b>ensures</b> result.value == coin1.value + coin2.value;
</code></pre>



<a name="0x1_Libra_Specification_deposit"></a>

### Function `deposit`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_deposit">deposit</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, check: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Libra_DepositAbortsIf">DepositAbortsIf</a>&lt;CoinType&gt;;
<b>ensures</b> coin.value == <b>old</b>(coin.value) + check.value;
</code></pre>




<a name="0x1_Libra_DepositAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_DepositAbortsIf">DepositAbortsIf</a>&lt;CoinType&gt; {
    coin: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;;
    check: <a href="#0x1_Libra">Libra</a>&lt;CoinType&gt;;
    <b>aborts_if</b> coin.value + check.value &gt; MAX_U64 with Errors::LIMIT_EXCEEDED;
}
</code></pre>



<a name="0x1_Libra_Specification_destroy_zero"></a>

### Function `destroy_zero`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(coin: <a href="#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>




<pre><code>pragma opaque;
<b>aborts_if</b> coin.value &gt; 0 with Errors::INVALID_ARGUMENT;
</code></pre>



<a name="0x1_Libra_Specification_register_currency"></a>

### Function `register_currency`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(lr_account: &signer, to_lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>, is_synthetic: bool, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;): (<a href="#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;, <a href="#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_Libra_RegisterCurrencyAbortsIf">RegisterCurrencyAbortsIf</a>&lt;CoinType&gt;;
<b>ensures</b> <a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;();
<b>ensures</b> <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value == 0;
</code></pre>




<a name="0x1_Libra_RegisterCurrencyAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_RegisterCurrencyAbortsIf">RegisterCurrencyAbortsIf</a>&lt;CoinType&gt; {
    lr_account: signer;
    currency_code: vector&lt;u8&gt;;
    scaling_factor: u64;
}
</code></pre>


Must abort if the signer does not have the LibraRoot role [B17].


<pre><code><b>schema</b> <a href="#0x1_Libra_RegisterCurrencyAbortsIf">RegisterCurrencyAbortsIf</a>&lt;CoinType&gt; {
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
    <b>aborts_if</b> scaling_factor == 0 || scaling_factor &gt; MAX_SCALING_FACTOR with Errors::INVALID_ARGUMENT;
    <b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotCurrencyInfo">CoreAddresses::AbortsIfNotCurrencyInfo</a>{account: lr_account};
    <b>aborts_if</b> exists&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account))
        with Errors::ALREADY_PUBLISHED;
    <b>include</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf">RegisteredCurrencies::AddCurrencyCodeAbortsIf</a>;
}
</code></pre>



<a name="0x1_Libra_Specification_register_SCS_currency"></a>

### Function `register_SCS_currency`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_register_SCS_currency">register_SCS_currency</a>&lt;CoinType&gt;(lr_account: &signer, tc_account: &signer, to_lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(tc_account)) with Errors::ALREADY_PUBLISHED;
<b>include</b> <a href="#0x1_Libra_RegisterCurrencyAbortsIf">RegisterCurrencyAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="#0x1_Libra_PublishBurnCapAbortsIfs">PublishBurnCapAbortsIfs</a>&lt;CoinType&gt;{account: tc_account};
<b>ensures</b> <a href="#0x1_Libra_spec_has_mint_capability">spec_has_mint_capability</a>&lt;CoinType&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(tc_account));
</code></pre>


Returns the market cap of CoinType.


<a name="0x1_Libra_spec_market_cap"></a>


<pre><code><b>define</b> <a href="#0x1_Libra_spec_market_cap">spec_market_cap</a>&lt;CoinType&gt;(): u128 {
<b>global</b>&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).total_value
}
</code></pre>



<a name="0x1_Libra_Specification_approx_lbr_for_value"></a>

### Function `approx_lbr_for_value`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value: u64): u64
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Libra_ApproxLbrForValueAbortsIf">ApproxLbrForValueAbortsIf</a>&lt;FromCoinType&gt;;
<b>ensures</b> result == <a href="#0x1_Libra_spec_approx_lbr_for_value">spec_approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value);
</code></pre>




<a name="0x1_Libra_ApproxLbrForValueAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_ApproxLbrForValueAbortsIf">ApproxLbrForValueAbortsIf</a>&lt;CoinType&gt; {
    from_value: num;
    <b>include</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;CoinType&gt;;
    <a name="0x1_Libra_lbr_exchange_rate$53"></a>
    <b>let</b> lbr_exchange_rate = <a href="#0x1_Libra_spec_lbr_exchange_rate">spec_lbr_exchange_rate</a>&lt;CoinType&gt;();
    <b>include</b> <a href="FixedPoint32.md#0x1_FixedPoint32_MultiplyAbortsIf">FixedPoint32::MultiplyAbortsIf</a>{val: from_value, multiplier: lbr_exchange_rate};
}
</code></pre>




<a name="0x1_Libra_spec_scaling_factor"></a>


<pre><code><b>define</b> <a href="#0x1_Libra_spec_scaling_factor">spec_scaling_factor</a>&lt;CoinType&gt;(): u64 {
<b>global</b>&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).scaling_factor
}
</code></pre>



<a name="0x1_Libra_Specification_currency_code"></a>

### Function `currency_code`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_currency_code">currency_code</a>&lt;CoinType&gt;(): vector&lt;u8&gt;
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;CoinType&gt;;
<b>ensures</b> result == <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().currency_code;
</code></pre>



<a name="0x1_Libra_Specification_update_lbr_exchange_rate"></a>

### Function `update_lbr_exchange_rate`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_update_lbr_exchange_rate">update_lbr_exchange_rate</a>&lt;FromCoinType&gt;(tc_account: &signer, lbr_exchange_rate: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>)
</code></pre>




<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;FromCoinType&gt;;
<b>ensures</b> <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;FromCoinType&gt;().to_lbr_exchange_rate == lbr_exchange_rate;
</code></pre>



<a name="0x1_Libra_Specification_assert_is_currency"></a>

### Function `assert_is_currency`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Libra_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;()
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_Libra_AbortsIfNoCurrency"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;CoinType&gt; {
    <b>aborts_if</b> !<a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;() with Errors::NOT_PUBLISHED;
}
</code></pre>




<a name="0x1_Libra_AbortsIfNoSCSCurrency"></a>


<pre><code><b>schema</b> <a href="#0x1_Libra_AbortsIfNoSCSCurrency">AbortsIfNoSCSCurrency</a>&lt;CoinType&gt; {
    <b>include</b> <a href="#0x1_Libra_AbortsIfNoCurrency">AbortsIfNoCurrency</a>&lt;CoinType&gt;;
    <b>aborts_if</b> !<a href="#0x1_Libra_spec_is_SCS_currency">spec_is_SCS_currency</a>&lt;CoinType&gt;() with Errors::INVALID_STATE;
}
</code></pre>


**************** MODULE SPECIFICATION ****************

<a name="0x1_Libra_@Module_Specification"></a>

### Module Specification


Verify all functions in this module.


<pre><code>pragma verify = <b>true</b>;
</code></pre>



Checks whether currency is registered. Mirrors
<code><a href="#0x1_Libra_is_currency">Self::is_currency</a>&lt;CoinType&gt;</code>.


<a name="0x1_Libra_spec_is_currency"></a>


<pre><code><b>define</b> <a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;(): bool {
    exists&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>


Returns currency information.


<a name="0x1_Libra_spec_currency_info"></a>


<pre><code><b>define</b> <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;(): <a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt; {
    <b>global</b>&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>


Specification version of
<code><a href="#0x1_Libra_approx_lbr_for_value">Self::approx_lbr_for_value</a></code>.


<a name="0x1_Libra_spec_approx_lbr_for_value"></a>


<pre><code><b>define</b> <a href="#0x1_Libra_spec_approx_lbr_for_value">spec_approx_lbr_for_value</a>&lt;CoinType&gt;(value: num):  num {
    <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">FixedPoint32::spec_multiply_u64</a>(value, <a href="#0x1_Libra_spec_lbr_exchange_rate">spec_lbr_exchange_rate</a>&lt;CoinType&gt;())
}
<a name="0x1_Libra_spec_lbr_exchange_rate"></a>
<b>define</b> <a href="#0x1_Libra_spec_lbr_exchange_rate">spec_lbr_exchange_rate</a>&lt;CoinType&gt;(): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> {
    <b>global</b>&lt;<a href="#0x1_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).to_lbr_exchange_rate
}
<a name="0x1_Libra_spec_is_SCS_currency"></a>
<b>define</b> <a href="#0x1_Libra_spec_is_SCS_currency">spec_is_SCS_currency</a>&lt;CoinType&gt;(): bool {
    <a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;() && !<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().is_synthetic
}
</code></pre>


Checks whether the currency has a mint capability.  This is only relevant for
SCS coins


<a name="0x1_Libra_spec_has_mint_capability"></a>


<pre><code><b>define</b> <a href="#0x1_Libra_spec_has_mint_capability">spec_has_mint_capability</a>&lt;CoinType&gt;(addr1: address): bool {
    exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(addr1)
}
</code></pre>



<a name="0x1_Libra_@Minting"></a>

#### Minting

For an SCS coin, the mint capability cannot move or disappear.
TODO: Specify that they're published at the one true treasurycompliance address?

If an address has a mint capability, it is an SCS currency.


<pre><code><b>invariant</b> [<b>global</b>]
    forall coin_type: type, addr3: address where <a href="#0x1_Libra_spec_has_mint_capability">spec_has_mint_capability</a>&lt;coin_type&gt;(addr3):
        <a href="#0x1_Libra_spec_is_SCS_currency">spec_is_SCS_currency</a>&lt;coin_type&gt;();
</code></pre>


If there is a pending offer for a mint capability, the coin_type is an SCS currency and
there are no published Mint Capabilities. (This is the state after register_SCS_currency_start)
> TODO: this invariant seems to be broken, as it has no premise regarding pending offers.


<pre><code><b>invariant</b> [<b>global</b>, deactivated]
    forall coin_type: type :
        <a href="#0x1_Libra_spec_is_SCS_currency">spec_is_SCS_currency</a>&lt;coin_type&gt;()
        && (forall addr3: address : !<a href="#0x1_Libra_spec_has_mint_capability">spec_has_mint_capability</a>&lt;coin_type&gt;(addr3));
<b>invariant</b> [<b>global</b>, isolated]
    forall coin_type: type where <a href="#0x1_Libra_spec_is_SCS_currency">spec_is_SCS_currency</a>&lt;coin_type&gt;():
        forall addr1: address, addr2: address
             where exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;coin_type&gt;&gt;(addr1) && exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;coin_type&gt;&gt;(addr2):
                  addr1 == addr2;
<b>invariant</b> <b>update</b> [<b>global</b>]
    forall coin_type: type:
        forall addr1: address where <b>old</b>(exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;coin_type&gt;&gt;(addr1)):
            exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;coin_type&gt;&gt;(addr1);
<b>invariant</b> [<b>global</b>]
    forall coin_type: type:
        forall addr1: address where exists&lt;<a href="#0x1_Libra_MintCapability">MintCapability</a>&lt;coin_type&gt;&gt;(addr1):
             <a href="Roles.md#0x1_Roles_spec_has_treasury_compliance_role_addr">Roles::spec_has_treasury_compliance_role_addr</a>(addr1);
</code></pre>



<a name="0x1_Libra_@Conservation_of_currency"></a>

#### Conservation of currency

TODO (dd): Unfortunately, improvements to the memory model have made it
difficult to compute sum_of_coin_values correctly. We will need
another approach for expressing this invariant.
TODO (dd): It would be great if we could prove that there is never a coin or a set of coins whose
aggregate value exceeds the CoinInfo.total_value.  However, that property involves summations over
all resources and is beyond the capabilities of the specification logic or the prover, currently.

The permission "MintCurrency(type)" is granted to TreasuryCompliance [B11].


<pre><code><b>apply</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account} <b>to</b> <a href="#0x1_Libra_register_SCS_currency">register_SCS_currency</a>&lt;CoinType&gt;;
</code></pre>


The permission "BurnCurrency(type)" is granted to TreasuryCompliance [B12].


<pre><code><b>apply</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account} <b>to</b> <a href="#0x1_Libra_register_SCS_currency">register_SCS_currency</a>&lt;CoinType&gt;;
</code></pre>


The permission "PreburnCurrency(type)" is granted to DesignatedDealer [B13].


<pre><code><b>apply</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDesignatedDealer">Roles::AbortsIfNotDesignatedDealer</a> <b>to</b> <a href="#0x1_Libra_publish_preburn_to_account">publish_preburn_to_account</a>&lt;CoinType&gt;;
</code></pre>


The permission "UpdateExchangeRate(type)" is granted to TreasuryCompliance [B14].


<pre><code><b>apply</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account} <b>to</b> <a href="#0x1_Libra_update_lbr_exchange_rate">update_lbr_exchange_rate</a>&lt;FromCoinType&gt;;
</code></pre>




<a name="0x1_Libra_TotalValueNotIncrease"></a>

The total amount of currency does not increase.


<pre><code><b>schema</b> <a href="#0x1_Libra_TotalValueNotIncrease">TotalValueNotIncrease</a>&lt;CoinType&gt; {
    <b>ensures</b> <b>old</b>(<a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;())
        ==&gt; <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value &lt;= <b>old</b>(<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value);
}
</code></pre>



Only mint functions can increase the total amount of currency [B11].


<pre><code><b>apply</b> <a href="#0x1_Libra_TotalValueNotIncrease">TotalValueNotIncrease</a>&lt;CoinType&gt; <b>to</b> *&lt;CoinType&gt;
    <b>except</b> <a href="#0x1_Libra_mint">mint</a>&lt;CoinType&gt;, <a href="#0x1_Libra_mint_with_capability">mint_with_capability</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_Libra_TotalValueNotDecrease"></a>

The total amount of currency does not decrease.


<pre><code><b>schema</b> <a href="#0x1_Libra_TotalValueNotDecrease">TotalValueNotDecrease</a>&lt;CoinType&gt; {
    <b>ensures</b> <b>old</b>(<a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;())
        ==&gt; <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value &gt;= <b>old</b>(<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value);
}
</code></pre>



Only burn functions can decrease the total amount of currency [B12].


<pre><code><b>apply</b> <a href="#0x1_Libra_TotalValueNotDecrease">TotalValueNotDecrease</a>&lt;CoinType&gt; <b>to</b> *&lt;CoinType&gt;
    <b>except</b> <a href="#0x1_Libra_burn">burn</a>&lt;CoinType&gt;, <a href="#0x1_Libra_burn_with_capability">burn_with_capability</a>&lt;CoinType&gt;, <a href="#0x1_Libra_burn_with_resource_cap">burn_with_resource_cap</a>&lt;CoinType&gt;,
    <a href="#0x1_Libra_burn_now">burn_now</a>&lt;CoinType&gt;;
</code></pre>


Only burn functions can decrease the preburn value of currency [B12].


<a name="0x1_Libra_PreburnValueNotIncrease"></a>

The preburn value of currency does not increase.


<pre><code><b>schema</b> <a href="#0x1_Libra_PreburnValueNotIncrease">PreburnValueNotIncrease</a>&lt;CoinType&gt; {
    <b>ensures</b> <b>old</b>(<a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;())
        ==&gt; <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().<a href="#0x1_Libra_preburn_value">preburn_value</a> &lt;= <b>old</b>(<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_Libra_PreburnValueNotDecrease">PreburnValueNotDecrease</a>&lt;CoinType&gt; <b>to</b> *&lt;CoinType&gt;
    <b>except</b> <a href="#0x1_Libra_burn">burn</a>&lt;CoinType&gt;, <a href="#0x1_Libra_burn_with_capability">burn_with_capability</a>&lt;CoinType&gt;, <a href="#0x1_Libra_burn_with_resource_cap">burn_with_resource_cap</a>&lt;CoinType&gt;,
    <a href="#0x1_Libra_burn_now">burn_now</a>&lt;CoinType&gt;, <a href="#0x1_Libra_cancel_burn">cancel_burn</a>&lt;CoinType&gt;, <a href="#0x1_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;CoinType&gt;;
</code></pre>


Only preburn functions can increase the preburn value of currency [B13].


<a name="0x1_Libra_PreburnValueNotDecrease"></a>

The the preburn value of currency does not decrease.


<pre><code><b>schema</b> <a href="#0x1_Libra_PreburnValueNotDecrease">PreburnValueNotDecrease</a>&lt;CoinType&gt; {
    <b>ensures</b> <b>old</b>(<a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;())
        ==&gt; <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value &gt;= <b>old</b>(<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_Libra_PreburnValueNotIncrease">PreburnValueNotIncrease</a>&lt;CoinType&gt; <b>to</b> *&lt;CoinType&gt;
    <b>except</b> <a href="#0x1_Libra_preburn_to">preburn_to</a>&lt;CoinType&gt;, <a href="#0x1_Libra_preburn_with_resource">preburn_with_resource</a>&lt;CoinType&gt;;
</code></pre>


Only update_lbr_exchange_rate can change the exchange rate [B14].


<a name="0x1_Libra_ExchangeRateRemainsSame"></a>

The exchange rate to LBR stays constant.


<pre><code><b>schema</b> <a href="#0x1_Libra_ExchangeRateRemainsSame">ExchangeRateRemainsSame</a>&lt;CoinType&gt; {
    <b>ensures</b> <b>old</b>(<a href="#0x1_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;())
        ==&gt; <a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().to_lbr_exchange_rate == <b>old</b>(<a href="#0x1_Libra_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().to_lbr_exchange_rate);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_Libra_ExchangeRateRemainsSame">ExchangeRateRemainsSame</a>&lt;CoinType&gt; <b>to</b> *&lt;CoinType&gt;
    <b>except</b> <a href="#0x1_Libra_update_lbr_exchange_rate">update_lbr_exchange_rate</a>&lt;CoinType&gt;;
</code></pre>
