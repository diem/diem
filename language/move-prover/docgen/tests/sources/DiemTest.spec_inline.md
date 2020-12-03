
<a name="0x1_DiemTest"></a>

# Module `0x1::DiemTest`

The <code><a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a></code> module describes the concept of a coin in the Diem framework. It introduces the
resource <code>Diem::Diem&lt;CoinType&gt;</code>, representing a coin of given coin type.
The module defines functions operating on coins as well as functionality like
minting and burning of coins.


-  [Resource `RegisterNewCurrency`](#0x1_DiemTest_RegisterNewCurrency)
-  [Resource `Diem`](#0x1_DiemTest_Diem)
-  [Resource `MintCapability`](#0x1_DiemTest_MintCapability)
-  [Resource `BurnCapability`](#0x1_DiemTest_BurnCapability)
-  [Struct `MintEvent`](#0x1_DiemTest_MintEvent)
-  [Struct `BurnEvent`](#0x1_DiemTest_BurnEvent)
-  [Struct `PreburnEvent`](#0x1_DiemTest_PreburnEvent)
-  [Struct `CancelBurnEvent`](#0x1_DiemTest_CancelBurnEvent)
-  [Struct `ToXDXExchangeRateUpdateEvent`](#0x1_DiemTest_ToXDXExchangeRateUpdateEvent)
-  [Resource `CurrencyInfo`](#0x1_DiemTest_CurrencyInfo)
-  [Resource `Preburn`](#0x1_DiemTest_Preburn)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_DiemTest_initialize)
-  [Function `publish_burn_capability`](#0x1_DiemTest_publish_burn_capability)
-  [Function `mint`](#0x1_DiemTest_mint)
-  [Function `burn`](#0x1_DiemTest_burn)
-  [Function `cancel_burn`](#0x1_DiemTest_cancel_burn)
-  [Function `mint_with_capability`](#0x1_DiemTest_mint_with_capability)
-  [Function `preburn_with_resource`](#0x1_DiemTest_preburn_with_resource)
-  [Function `create_preburn`](#0x1_DiemTest_create_preburn)
-  [Function `publish_preburn_to_account`](#0x1_DiemTest_publish_preburn_to_account)
-  [Function `preburn_to`](#0x1_DiemTest_preburn_to)
-  [Function `burn_with_capability`](#0x1_DiemTest_burn_with_capability)
-  [Function `burn_with_resource_cap`](#0x1_DiemTest_burn_with_resource_cap)
-  [Function `cancel_burn_with_capability`](#0x1_DiemTest_cancel_burn_with_capability)
-  [Function `remove_burn_capability`](#0x1_DiemTest_remove_burn_capability)
-  [Function `preburn_value`](#0x1_DiemTest_preburn_value)
-  [Function `zero`](#0x1_DiemTest_zero)
-  [Function `value`](#0x1_DiemTest_value)
-  [Function `split`](#0x1_DiemTest_split)
-  [Function `withdraw`](#0x1_DiemTest_withdraw)
-  [Function `withdraw_all`](#0x1_DiemTest_withdraw_all)
-  [Function `join`](#0x1_DiemTest_join)
-  [Function `deposit`](#0x1_DiemTest_deposit)
-  [Function `destroy_zero`](#0x1_DiemTest_destroy_zero)
-  [Function `register_currency`](#0x1_DiemTest_register_currency)
-  [Function `register_SCS_currency`](#0x1_DiemTest_register_SCS_currency)
-  [Function `market_cap`](#0x1_DiemTest_market_cap)
-  [Function `approx_xdx_for_value`](#0x1_DiemTest_approx_xdx_for_value)
-  [Function `approx_xdx_for_coin`](#0x1_DiemTest_approx_xdx_for_coin)
-  [Function `is_currency`](#0x1_DiemTest_is_currency)
-  [Function `is_SCS_currency`](#0x1_DiemTest_is_SCS_currency)
-  [Function `is_synthetic_currency`](#0x1_DiemTest_is_synthetic_currency)
-  [Function `scaling_factor`](#0x1_DiemTest_scaling_factor)
-  [Function `fractional_part`](#0x1_DiemTest_fractional_part)
-  [Function `currency_code`](#0x1_DiemTest_currency_code)
-  [Function `update_xdx_exchange_rate`](#0x1_DiemTest_update_xdx_exchange_rate)
-  [Function `xdx_exchange_rate`](#0x1_DiemTest_xdx_exchange_rate)
-  [Function `update_minting_ability`](#0x1_DiemTest_update_minting_ability)
-  [Function `assert_is_currency`](#0x1_DiemTest_assert_is_currency)
-  [Function `assert_is_SCS_currency`](#0x1_DiemTest_assert_is_SCS_currency)
-  [Module Specification](#@Module_Specification_1)
    -  [Module Specification](#@Module_Specification_2)
        -  [Minting](#@Minting_3)
        -  [Conservation of currency](#@Conservation_of_currency_4)


<pre><code><b>use</b> <a href="">0x1::CoreAddresses</a>;
<b>use</b> <a href="">0x1::DiemTimestamp</a>;
<b>use</b> <a href="">0x1::Event</a>;
<b>use</b> <a href="">0x1::FixedPoint32</a>;
<b>use</b> <a href="">0x1::RegisteredCurrencies</a>;
<b>use</b> <a href="">0x1::Roles</a>;
<b>use</b> <a href="">0x1::Signer</a>;
</code></pre>



<a name="0x1_DiemTest_RegisterNewCurrency"></a>

## Resource `RegisterNewCurrency`



<pre><code><b>resource</b> <b>struct</b> <a href="DiemTest.md#0x1_DiemTest_RegisterNewCurrency">RegisterNewCurrency</a>
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

<a name="0x1_DiemTest_Diem"></a>

## Resource `Diem`

The <code><a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a></code> resource defines the Diem coin for each currency in
Diem. Each "coin" is coupled with a type <code>CoinType</code> specifying the
currency of the coin, and a <code>value</code> field specifying the value
of the coin (in the base units of the currency <code>CoinType</code>
and specified in the <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code> resource for that <code>CoinType</code>
published under the <code><a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code> account address).


<pre><code><b>resource</b> <b>struct</b> <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>value: u64</code>
</dt>
<dd>
 The value of this coin in the base units for <code>CoinType</code>
</dd>
</dl>


</details>

<details>
<summary>Specification</summary>

Account for updating <code><a href="DiemTest.md#0x1_DiemTest_sum_of_coin_values">sum_of_coin_values</a></code> when a <code><a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a></code> is packed or unpacked.


<pre><code><b>invariant</b> <b>pack</b> <a href="DiemTest.md#0x1_DiemTest_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; = <a href="DiemTest.md#0x1_DiemTest_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; + value;
<b>invariant</b> <b>unpack</b> <a href="DiemTest.md#0x1_DiemTest_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; = <a href="DiemTest.md#0x1_DiemTest_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; - value;
</code></pre>



</details>

<a name="0x1_DiemTest_MintCapability"></a>

## Resource `MintCapability`

The <code><a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a></code> resource defines a capability to allow minting
of coins of <code>CoinType</code> currency by the holder of this capability.
This capability is held only either by the <code><a href="_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()</code>
account or the <code>0x1::XDX</code> module (and <code><a href="_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>()</code> in testnet).


<pre><code><b>resource</b> <b>struct</b> <a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;
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

<a name="0x1_DiemTest_BurnCapability"></a>

## Resource `BurnCapability`

The <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a></code> resource defines a capability to allow coins
of <code>CoinType</code> currency to be burned by the holder of the
and the <code>0x1::XDX</code> module (and <code><a href="_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>()</code> in testnet).


<pre><code><b>resource</b> <b>struct</b> <a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
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

<a name="0x1_DiemTest_MintEvent"></a>

## Struct `MintEvent`

The <code>CurrencyRegistrationCapability</code> is a singleton resource
published under the <code><a href="_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>()</code> and grants
the capability to the <code>0x1::Diem</code> module to add currencies to the
<code><a href="">0x1::RegisteredCurrencies</a></code> on-chain config.
A <code><a href="DiemTest.md#0x1_DiemTest_MintEvent">MintEvent</a></code> is emitted every time a Diem coin is minted. This
contains the <code>amount</code> minted (in base units of the currency being
minted) along with the <code>currency_code</code> for the coin(s) being
minted, and that is defined in the <code>currency_code</code> field of the
<code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code> resource for the currency.


<pre><code><b>struct</b> <a href="DiemTest.md#0x1_DiemTest_MintEvent">MintEvent</a>
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
 ASCII encoded symbol for the coin type (e.g., "XDX")
</dd>
</dl>


</details>

<a name="0x1_DiemTest_BurnEvent"></a>

## Struct `BurnEvent`

A <code><a href="DiemTest.md#0x1_DiemTest_BurnEvent">BurnEvent</a></code> is emitted every time a non-synthetic[1] Diem coin is
burned. It contains the <code>amount</code> burned in base units for the
currency, along with the <code>currency_code</code> for the coins being burned
(and as defined in the <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code> resource for that currency).
It also contains the <code>preburn_address</code> from which the coin is
extracted for burning.
[1] As defined by the <code>is_synthetic</code> field in the <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code>
for that currency.


<pre><code><b>struct</b> <a href="DiemTest.md#0x1_DiemTest_BurnEvent">BurnEvent</a>
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
 ASCII encoded symbol for the coin type (e.g., "XDX")
</dd>
<dt>
<code>preburn_address: address</code>
</dt>
<dd>
 Address with the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource that stored the now-burned funds
</dd>
</dl>


</details>

<a name="0x1_DiemTest_PreburnEvent"></a>

## Struct `PreburnEvent`

A <code><a href="DiemTest.md#0x1_DiemTest_PreburnEvent">PreburnEvent</a></code> is emitted every time an <code>amount</code> of funds with
a coin type <code>currency_code</code> are moved to a <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource under
the account at the address <code>preburn_address</code>.


<pre><code><b>struct</b> <a href="DiemTest.md#0x1_DiemTest_PreburnEvent">PreburnEvent</a>
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
 ASCII encoded symbol for the coin type (e.g., "XDX")
</dd>
<dt>
<code>preburn_address: address</code>
</dt>
<dd>
 Address with the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource that now holds the funds
</dd>
</dl>


</details>

<a name="0x1_DiemTest_CancelBurnEvent"></a>

## Struct `CancelBurnEvent`

A <code><a href="DiemTest.md#0x1_DiemTest_CancelBurnEvent">CancelBurnEvent</a></code> is emitted every time funds of <code>amount</code> in a <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code>
resource at <code>preburn_address</code> is canceled (removed from the
preburn, but not burned). The currency of the funds is given by the
<code>currency_code</code> as defined in the <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code> for that currency.


<pre><code><b>struct</b> <a href="DiemTest.md#0x1_DiemTest_CancelBurnEvent">CancelBurnEvent</a>
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
 ASCII encoded symbol for the coin type (e.g., "XDX")
</dd>
<dt>
<code>preburn_address: address</code>
</dt>
<dd>
 Address of the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource that held the now-returned funds.
</dd>
</dl>


</details>

<a name="0x1_DiemTest_ToXDXExchangeRateUpdateEvent"></a>

## Struct `ToXDXExchangeRateUpdateEvent`

An <code><a href="DiemTest.md#0x1_DiemTest_ToXDXExchangeRateUpdateEvent">ToXDXExchangeRateUpdateEvent</a></code> is emitted every time the to-XDX exchange
rate for the currency given by <code>currency_code</code> is updated.


<pre><code><b>struct</b> <a href="DiemTest.md#0x1_DiemTest_ToXDXExchangeRateUpdateEvent">ToXDXExchangeRateUpdateEvent</a>
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
<code>new_to_xdx_exchange_rate: u64</code>
</dt>
<dd>
 The new on-chain to-XDX exchange rate between the
 <code>currency_code</code> currency and XDX. Represented in conversion
 between the (on-chain) base-units for the currency and microdiem.
</dd>
</dl>


</details>

<a name="0x1_DiemTest_CurrencyInfo"></a>

## Resource `CurrencyInfo`

The <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;</code> resource stores the various
pieces of information needed for a currency (<code>CoinType</code>) that is
registered on-chain. This resource _must_ be published under the
address given by <code><a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code> in order for the registration of
<code>CoinType</code> as a recognized currency on-chain to be successful. At
the time of registration the <code><a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> and
<code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> capabilities are returned to the caller.
Unless they are specified otherwise the fields in this resource are immutable.


<pre><code><b>resource</b> <b>struct</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>total_value: u128</code>
</dt>
<dd>
 The total value for the currency represented by <code>CoinType</code>. Mutable.
</dd>
<dt>
<code>preburn_value: u64</code>
</dt>
<dd>
 Value of funds that are in the process of being burned.  Mutable.
</dd>
<dt>
<code>to_xdx_exchange_rate: <a href="_FixedPoint32">FixedPoint32::FixedPoint32</a></code>
</dt>
<dd>
 The (rough) exchange rate from <code>CoinType</code> to <code>XDX</code>. Mutable.
</dd>
<dt>
<code>is_synthetic: bool</code>
</dt>
<dd>
 Holds whether or not this currency is synthetic (contributes to the
 off-chain reserve) or not. An example of such a synthetic
currency would be the XDX.
</dd>
<dt>
<code>scaling_factor: u64</code>
</dt>
<dd>
 The scaling factor for the coin (i.e. the amount to multiply by
 to get to the human-readable representation for this currency).
 e.g. 10^6 for <code>XUS</code>

 > TODO(wrwg): should the above be "to divide by"?
</dd>
<dt>
<code>fractional_part: u64</code>
</dt>
<dd>
 The smallest fractional part (number of decimal places) to be
 used in the human-readable representation for the currency (e.g.
 10^2 for <code>XUS</code> cents)
</dd>
<dt>
<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 The code symbol for this <code>CoinType</code>. ASCII encoded.
 e.g. for "XDX" this is x"4C4252". No character limit.
</dd>
<dt>
<code>can_mint: bool</code>
</dt>
<dd>
 We may want to disable the ability to mint further coins of a
 currency while that currency is still around. This allows us to
 keep the currency in circulation while disallowing further
 creation of coins in the <code>CoinType</code> currency. Mutable.
</dd>
<dt>
<code>mint_events: <a href="_EventHandle">Event::EventHandle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_MintEvent">DiemTest::MintEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for minting and where <code><a href="DiemTest.md#0x1_DiemTest_MintEvent">MintEvent</a></code>s will be emitted.
</dd>
<dt>
<code>burn_events: <a href="_EventHandle">Event::EventHandle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_BurnEvent">DiemTest::BurnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for burning, and where <code><a href="DiemTest.md#0x1_DiemTest_BurnEvent">BurnEvent</a></code>s will be emitted.
</dd>
<dt>
<code>preburn_events: <a href="_EventHandle">Event::EventHandle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_PreburnEvent">DiemTest::PreburnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for preburn requests, and where all
 <code><a href="DiemTest.md#0x1_DiemTest_PreburnEvent">PreburnEvent</a></code>s for this <code>CoinType</code> will be emitted.
</dd>
<dt>
<code>cancel_burn_events: <a href="_EventHandle">Event::EventHandle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_CancelBurnEvent">DiemTest::CancelBurnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for all cancelled preburn requests for this
 <code>CoinType</code>.
</dd>
<dt>
<code>exchange_rate_update_events: <a href="_EventHandle">Event::EventHandle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_ToXDXExchangeRateUpdateEvent">DiemTest::ToXDXExchangeRateUpdateEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for emiting exchange rate change events
</dd>
</dl>


</details>

<a name="0x1_DiemTest_Preburn"></a>

## Resource `Preburn`

A holding area where funds that will subsequently be burned wait while their underlying
assets are moved off-chain.
This resource can only be created by the holder of a <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a></code>. An account that
contains this address has the authority to initiate a burn request. A burn request can be
resolved by the holder of a <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a></code> by either (1) burning the funds, or (2)
returning the funds to the account that initiated the burn request.
Concurrent preburn requests are not allowed, only one request (in to_burn) can be handled at any time.


<pre><code><b>resource</b> <b>struct</b> <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>to_burn: <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;</code>
</dt>
<dd>
 A single pending burn amount.
 There is no pending burn request if the value in to_burn is 0
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemTest_ENOT_GENESIS"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_ENOT_GENESIS">ENOT_GENESIS</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemTest_EAMOUNT_EXCEEDS_COIN_VALUE"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_EAMOUNT_EXCEEDS_COIN_VALUE">EAMOUNT_EXCEEDS_COIN_VALUE</a>: u64 = 5;
</code></pre>



<a name="0x1_DiemTest_EDESTRUCTION_OF_NONZERO_COIN"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_EDESTRUCTION_OF_NONZERO_COIN">EDESTRUCTION_OF_NONZERO_COIN</a>: u64 = 6;
</code></pre>



<a name="0x1_DiemTest_EDOES_NOT_HAVE_DIEM_ROOT_ROLE"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_EDOES_NOT_HAVE_DIEM_ROOT_ROLE">EDOES_NOT_HAVE_DIEM_ROOT_ROLE</a>: u64 = 9;
</code></pre>



<a name="0x1_DiemTest_EDOES_NOT_HAVE_TREASURY_COMPLIANCE_ROLE"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_EDOES_NOT_HAVE_TREASURY_COMPLIANCE_ROLE">EDOES_NOT_HAVE_TREASURY_COMPLIANCE_ROLE</a>: u64 = 10;
</code></pre>



<a name="0x1_DiemTest_EINVALID_SINGLETON_ADDRESS"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_EINVALID_SINGLETON_ADDRESS">EINVALID_SINGLETON_ADDRESS</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemTest_EIS_SYNTHETIC_CURRENCY"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_EIS_SYNTHETIC_CURRENCY">EIS_SYNTHETIC_CURRENCY</a>: u64 = 4;
</code></pre>



<a name="0x1_DiemTest_EMINTING_NOT_ALLOWED"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_EMINTING_NOT_ALLOWED">EMINTING_NOT_ALLOWED</a>: u64 = 3;
</code></pre>



<a name="0x1_DiemTest_ENOT_AN_SCS_CURRENCY"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_ENOT_AN_SCS_CURRENCY">ENOT_AN_SCS_CURRENCY</a>: u64 = 8;
</code></pre>



<a name="0x1_DiemTest_ENOT_A_REGISTERED_CURRENCY"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_ENOT_A_REGISTERED_CURRENCY">ENOT_A_REGISTERED_CURRENCY</a>: u64 = 7;
</code></pre>



<a name="0x1_DiemTest_ENOT_TREASURY_COMPLIANCE"></a>



<pre><code><b>const</b> <a href="DiemTest.md#0x1_DiemTest_ENOT_TREASURY_COMPLIANCE">ENOT_TREASURY_COMPLIANCE</a>: u64 = 2;
</code></pre>



<a name="0x1_DiemTest_initialize"></a>

## Function `initialize`

Grants the <code><a href="DiemTest.md#0x1_DiemTest_RegisterNewCurrency">RegisterNewCurrency</a></code> privilege to
the calling account as long as it has the correct role (TC).
Aborts if <code>account</code> does not have a <code>RoleId</code> that corresponds with
the treacury compliance role.
Initialization of the <code><a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a></code> module; initializes the set of
registered currencies in the <code><a href="">0x1::RegisteredCurrencies</a></code> on-chain
config, and publishes the <code>CurrencyRegistrationCapability</code> under the
<code><a href="_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>()</code>. This can only be called from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_initialize">initialize</a>(config_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_initialize">initialize</a>(
    config_account: &signer,
) {
    <b>assert</b>(<a href="_is_genesis">DiemTimestamp::is_genesis</a>(), <a href="DiemTest.md#0x1_DiemTest_ENOT_GENESIS">ENOT_GENESIS</a>);
    // Operational constraint
    <b>assert</b>(
        <a href="_address_of">Signer::address_of</a>(config_account) == <a href="_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>(),
        <a href="DiemTest.md#0x1_DiemTest_EINVALID_SINGLETON_ADDRESS">EINVALID_SINGLETON_ADDRESS</a>
    );
    <a href="_initialize">RegisteredCurrencies::initialize</a>(config_account);
}
</code></pre>



</details>

<a name="0x1_DiemTest_publish_burn_capability"></a>

## Function `publish_burn_capability`

Publishes the <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a></code> <code>cap</code> for the <code>CoinType</code> currency under <code>account</code>. <code>CoinType</code>
must be a registered currency type.
The caller must pass a <code>TreasuryComplianceRole</code> capability.
TODO (dd): I think there is a multiple signer problem here.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_publish_burn_capability">publish_burn_capability</a>&lt;CoinType&gt;(account: &signer, cap: <a href="DiemTest.md#0x1_DiemTest_BurnCapability">DiemTest::BurnCapability</a>&lt;CoinType&gt;, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_publish_burn_capability">publish_burn_capability</a>&lt;CoinType&gt;(
    account: &signer,
    cap: <a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;,
    tc_account: &signer,
) {
    <b>assert</b>(<a href="_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(tc_account), <a href="DiemTest.md#0x1_DiemTest_ENOT_TREASURY_COMPLIANCE">ENOT_TREASURY_COMPLIANCE</a>);
    <a href="DiemTest.md#0x1_DiemTest_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    move_to(account, cap)
}
</code></pre>



</details>

<a name="0x1_DiemTest_mint"></a>

## Function `mint`

Mints <code>amount</code> coins. The <code>account</code> must hold a
<code><a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> at the top-level in order for this call
to be successful, and will fail with <code>MISSING_DATA</code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_mint">mint</a>&lt;CoinType&gt;(account: &signer, value: u64): <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_mint">mint</a>&lt;CoinType&gt;(account: &signer, value: u64): <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>, <a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a> {
    <a href="DiemTest.md#0x1_DiemTest_mint_with_capability">mint_with_capability</a>(
        value,
        borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(<a href="_address_of">Signer::address_of</a>(account))
    )
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(<a href="_spec_address_of">Signer::spec_address_of</a>(account));
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_MintAbortsIf">MintAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_MintEnsures">MintEnsures</a>&lt;CoinType&gt;;
</code></pre>



</details>

<a name="0x1_DiemTest_burn"></a>

## Function `burn`

Burns the coins currently held in the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource held under <code>preburn_address</code>.
Calls to this functions will fail if the <code>account</code> does not have a
published <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a></code> for the <code>CoinType</code> published under it.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_burn">burn</a>&lt;CoinType&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_burn">burn</a>&lt;CoinType&gt;(
    account: &signer,
    preburn_address: address
) <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>, <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>, <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a> {
    <a href="DiemTest.md#0x1_DiemTest_burn_with_capability">burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="_address_of">Signer::address_of</a>(account))
    )
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> verify=<b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



</details>

<a name="0x1_DiemTest_cancel_burn"></a>

## Function `cancel_burn`

Cancels the current burn request in the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource held
under the <code>preburn_address</code>, and returns the coins.
Calls to this will fail if the sender does not have a published
<code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>, or if there is no preburn request
outstanding in the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource under <code>preburn_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_cancel_burn">cancel_burn</a>&lt;CoinType&gt;(account: &signer, preburn_address: address): <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_cancel_burn">cancel_burn</a>&lt;CoinType&gt;(
    account: &signer,
    preburn_address: address
): <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt; <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>, <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>, <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a> {
    <a href="DiemTest.md#0x1_DiemTest_cancel_burn_with_capability">cancel_burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="_address_of">Signer::address_of</a>(account))
    )
}
</code></pre>



</details>

<a name="0x1_DiemTest_mint_with_capability"></a>

## Function `mint_with_capability`

Mint a new <code><a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a></code> coin of <code>CoinType</code> currency worth <code>value</code>. The
caller must have a reference to a <code><a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;</code>. Only
the treasury compliance account or the <code>0x1::XDX</code> module can acquire such a
reference.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_mint_with_capability">mint_with_capability</a>&lt;CoinType&gt;(value: u64, _capability: &<a href="DiemTest.md#0x1_DiemTest_MintCapability">DiemTest::MintCapability</a>&lt;CoinType&gt;): <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_mint_with_capability">mint_with_capability</a>&lt;CoinType&gt;(
    value: u64,
    _capability: &<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;
): <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt; <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <a href="DiemTest.md#0x1_DiemTest_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <b>let</b> currency_code = <a href="DiemTest.md#0x1_DiemTest_currency_code">currency_code</a>&lt;CoinType&gt;();
    // <b>update</b> market cap <b>resource</b> <b>to</b> reflect minting
    <b>let</b> info = borrow_global_mut&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    <b>assert</b>(info.can_mint, <a href="DiemTest.md#0x1_DiemTest_EMINTING_NOT_ALLOWED">EMINTING_NOT_ALLOWED</a>);
    info.total_value = info.total_value + (value <b>as</b> u128);
    // don't emit mint events for synthetic currenices
    <b>if</b> (!info.is_synthetic) {
        <a href="_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.mint_events,
            <a href="DiemTest.md#0x1_DiemTest_MintEvent">MintEvent</a>{
                amount: value,
                currency_code,
            }
        );
    };

    <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt; { value }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemTest.md#0x1_DiemTest_MintAbortsIf">MintAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_MintEnsures">MintEnsures</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_DiemTest_MintAbortsIf"></a>


<pre><code><b>schema</b> <a href="DiemTest.md#0x1_DiemTest_MintAbortsIf">MintAbortsIf</a>&lt;CoinType&gt; {
    value: u64;
    <b>aborts_if</b> !<a href="DiemTest.md#0x1_DiemTest_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;();
    <b>aborts_if</b> !<a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().can_mint;
    <b>aborts_if</b> <a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value + value &gt; max_u128();
}
</code></pre>




<a name="0x1_DiemTest_MintEnsures"></a>


<pre><code><b>schema</b> <a href="DiemTest.md#0x1_DiemTest_MintEnsures">MintEnsures</a>&lt;CoinType&gt; {
    value: u64;
    result: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;;
    <b>ensures</b> <a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value
                == <b>old</b>(<a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value) + value;
    <b>ensures</b> result.value == value;
}
</code></pre>



</details>

<a name="0x1_DiemTest_preburn_with_resource"></a>

## Function `preburn_with_resource`

Add the <code>coin</code> to the <code>preburn</code> to_burn field in the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource
held at the address <code>preburn_address</code> if it is empty, otherwise raise
a PendingPreburn Error (code 6). Emits a <code><a href="DiemTest.md#0x1_DiemTest_PreburnEvent">PreburnEvent</a></code> to
the <code>preburn_events</code> event stream in the <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code> for the
<code>CoinType</code> passed in. However, if the currency being preburned is
<code>synthetic</code> then no <code><a href="DiemTest.md#0x1_DiemTest_PreburnEvent">PreburnEvent</a></code> event will be emitted.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_preburn_with_resource">preburn_with_resource</a>&lt;CoinType&gt;(coin: <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;, preburn: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Preburn">DiemTest::Preburn</a>&lt;CoinType&gt;, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_preburn_with_resource">preburn_with_resource</a>&lt;CoinType&gt;(
    coin: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;,
    preburn: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;,
    preburn_address: address,
) <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> coin_value = <a href="DiemTest.md#0x1_DiemTest_value">value</a>(&coin);
    // Throw <b>if</b> already occupied
    <b>assert</b>(<a href="DiemTest.md#0x1_DiemTest_value">value</a>(&preburn.to_burn) == 0, 6);
    <a href="DiemTest.md#0x1_DiemTest_deposit">deposit</a>(&<b>mut</b> preburn.to_burn, coin);
    <b>let</b> currency_code = <a href="DiemTest.md#0x1_DiemTest_currency_code">currency_code</a>&lt;CoinType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    info.preburn_value = info.preburn_value + coin_value;
    // don't emit preburn events for synthetic currencies
    <b>if</b> (!info.is_synthetic) {
        <a href="_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.preburn_events,
            <a href="DiemTest.md#0x1_DiemTest_PreburnEvent">PreburnEvent</a>{
                amount: coin_value,
                currency_code,
                preburn_address,
            }
        );
    };
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_PreburnAbortsIf">PreburnAbortsIf</a>&lt;CoinType&gt;;
<b>aborts_if</b> preburn.to_burn.value != 0;
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_PreburnEnsures">PreburnEnsures</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_DiemTest_PreburnAbortsIf"></a>


<pre><code><b>schema</b> <a href="DiemTest.md#0x1_DiemTest_PreburnAbortsIf">PreburnAbortsIf</a>&lt;CoinType&gt; {
    coin: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;;
    <b>aborts_if</b> !<a href="DiemTest.md#0x1_DiemTest_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;();
    <b>aborts_if</b> <a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value + coin.value &gt; max_u64();
}
</code></pre>




<a name="0x1_DiemTest_PreburnEnsures"></a>


<pre><code><b>schema</b> <a href="DiemTest.md#0x1_DiemTest_PreburnEnsures">PreburnEnsures</a>&lt;CoinType&gt; {
    coin: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;;
    preburn: <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;;
    <b>ensures</b> <a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value
                == <b>old</b>(<a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value) + coin.value;
}
</code></pre>



</details>

<a name="0x1_DiemTest_create_preburn"></a>

## Function `create_preburn`

Create a <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;</code> resource


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_create_preburn">create_preburn</a>&lt;CoinType&gt;(tc_account: &signer): <a href="DiemTest.md#0x1_DiemTest_Preburn">DiemTest::Preburn</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_create_preburn">create_preburn</a>&lt;CoinType&gt;(
    tc_account: &signer
): <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt; {
    <b>assert</b>(<a href="_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(tc_account), <a href="DiemTest.md#0x1_DiemTest_ENOT_TREASURY_COMPLIANCE">ENOT_TREASURY_COMPLIANCE</a>);
    <a href="DiemTest.md#0x1_DiemTest_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    // TODO (dd): consider adding an assertion here that to_burn &lt;= info.total_value
    // I don't think it can happen, but that may be difficult <b>to</b> prove.
    <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt; { to_burn: <a href="DiemTest.md#0x1_DiemTest_zero">zero</a>&lt;CoinType&gt;() }
}
</code></pre>



</details>

<a name="0x1_DiemTest_publish_preburn_to_account"></a>

## Function `publish_preburn_to_account`

Publishes a <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource under <code>account</code>. This function is
used for bootstrapping the designated dealer at account-creation
time, and the association TC account <code>creator</code> (at <code><a href="_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()</code>) is creating
this resource for the designated dealer.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_publish_preburn_to_account">publish_preburn_to_account</a>&lt;CoinType&gt;(account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_publish_preburn_to_account">publish_preburn_to_account</a>&lt;CoinType&gt;(
    account: &signer,
    tc_account: &signer
) <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>assert</b>(!<a href="DiemTest.md#0x1_DiemTest_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(), <a href="DiemTest.md#0x1_DiemTest_EIS_SYNTHETIC_CURRENCY">EIS_SYNTHETIC_CURRENCY</a>);
    move_to(account, <a href="DiemTest.md#0x1_DiemTest_create_preburn">create_preburn</a>&lt;CoinType&gt;(tc_account))
}
</code></pre>



</details>

<a name="0x1_DiemTest_preburn_to"></a>

## Function `preburn_to`

Sends <code>coin</code> to the preburn queue for <code>account</code>, where it will wait to either be burned
or returned to the balance of <code>account</code>.
Calls to this function will fail if <code>account</code> does not have a
<code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;</code> resource published under it.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_preburn_to">preburn_to</a>&lt;CoinType&gt;(account: &signer, coin: <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_preburn_to">preburn_to</a>&lt;CoinType&gt;(
    account: &signer, coin: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;) <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>, <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a> {
    <b>let</b> sender = <a href="_address_of">Signer::address_of</a>(account);
    <a href="DiemTest.md#0x1_DiemTest_preburn_with_resource">preburn_with_resource</a>(coin, borrow_global_mut&lt;<a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(sender), sender);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(<a href="_spec_address_of">Signer::spec_address_of</a>(account));
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_PreburnAbortsIf">PreburnAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_PreburnEnsures">PreburnEnsures</a>&lt;CoinType&gt;{preburn: <b>global</b>&lt;<a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(<a href="_spec_address_of">Signer::spec_address_of</a>(account))};
</code></pre>



</details>

<a name="0x1_DiemTest_burn_with_capability"></a>

## Function `burn_with_capability`

Permanently removes the coins held in the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource (in to_burn field)
stored at <code>preburn_address</code> and updates the market cap accordingly.
This function can only be called by the holder of a <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>.
Calls to this function will fail if the there is no <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;</code>
resource under <code>preburn_address</code>, or, if the preburn to_burn area for
<code>CoinType</code> is empty (error code 7).


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_burn_with_capability">burn_with_capability</a>&lt;CoinType&gt;(preburn_address: address, capability: &<a href="DiemTest.md#0x1_DiemTest_BurnCapability">DiemTest::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_burn_with_capability">burn_with_capability</a>&lt;CoinType&gt;(
    preburn_address: address,
    capability: &<a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
) <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>, <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a> {
    // destroy the coin in the preburn to_burn area
    <a href="DiemTest.md#0x1_DiemTest_burn_with_resource_cap">burn_with_resource_cap</a>(
        borrow_global_mut&lt;<a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address),
        preburn_address,
        capability
    )
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address);
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_BurnAbortsIf">BurnAbortsIf</a>&lt;CoinType&gt;{preburn: <b>global</b>&lt;<a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address)};
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_BurnEnsures">BurnEnsures</a>&lt;CoinType&gt;{preburn: <b>global</b>&lt;<a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address)};
</code></pre>



</details>

<a name="0x1_DiemTest_burn_with_resource_cap"></a>

## Function `burn_with_resource_cap`

Permanently removes the coins held in the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource (in to_burn field)
stored at <code>preburn_address</code> and updates the market cap accordingly.
This function can only be called by the holder of a <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>.
Calls to this function will fail if the there is no <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;</code>
resource under <code>preburn_address</code>, or, if the preburn to_burn area for
<code>CoinType</code> is empty (error code 7).


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_burn_with_resource_cap">burn_with_resource_cap</a>&lt;CoinType&gt;(preburn: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Preburn">DiemTest::Preburn</a>&lt;CoinType&gt;, preburn_address: address, _capability: &<a href="DiemTest.md#0x1_DiemTest_BurnCapability">DiemTest::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_burn_with_resource_cap">burn_with_resource_cap</a>&lt;CoinType&gt;(
    preburn: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;,
    preburn_address: address,
    _capability: &<a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
) <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> currency_code = <a href="DiemTest.md#0x1_DiemTest_currency_code">currency_code</a>&lt;CoinType&gt;();
    // Abort <b>if</b> no coin present in preburn area
    <b>assert</b>(preburn.to_burn.value &gt; 0, 7);
    // destroy the coin in <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a> area
    <b>let</b> <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a> { value } = <a href="DiemTest.md#0x1_DiemTest_withdraw_all">withdraw_all</a>&lt;CoinType&gt;(&<b>mut</b> preburn.to_burn);
    // <b>update</b> the market cap
    <b>let</b> info = borrow_global_mut&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    info.total_value = info.total_value - (value <b>as</b> u128);
    info.preburn_value = info.preburn_value - value;
    // don't emit burn events for synthetic currencies
    <b>if</b> (!info.is_synthetic) {
        <a href="_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.burn_events,
            <a href="DiemTest.md#0x1_DiemTest_BurnEvent">BurnEvent</a> {
                amount: value,
                currency_code,
                preburn_address,
            }
        );
    };
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemTest.md#0x1_DiemTest_BurnAbortsIf">BurnAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="DiemTest.md#0x1_DiemTest_BurnEnsures">BurnEnsures</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_DiemTest_BurnAbortsIf"></a>


<pre><code><b>schema</b> <a href="DiemTest.md#0x1_DiemTest_BurnAbortsIf">BurnAbortsIf</a>&lt;CoinType&gt; {
    preburn: <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;;
    <b>aborts_if</b> !<a href="DiemTest.md#0x1_DiemTest_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;();
    <b>aborts_if</b> preburn.to_burn.value == 0;
    <b>aborts_if</b> <a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value - preburn.to_burn.<a href="DiemTest.md#0x1_DiemTest_value">value</a> &lt; 0;
    <b>aborts_if</b> <a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value - preburn.to_burn.<a href="DiemTest.md#0x1_DiemTest_value">value</a> &lt; 0;
}
</code></pre>




<a name="0x1_DiemTest_BurnEnsures"></a>


<pre><code><b>schema</b> <a href="DiemTest.md#0x1_DiemTest_BurnEnsures">BurnEnsures</a>&lt;CoinType&gt; {
    preburn: <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;;
    <b>ensures</b> <a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value
            == <b>old</b>(<a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().total_value) - <b>old</b>(preburn.to_burn.value);
    <b>ensures</b> <a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value
            == <b>old</b>(<a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().preburn_value) - <b>old</b>(preburn.to_burn.value);
}
</code></pre>



</details>

<a name="0x1_DiemTest_cancel_burn_with_capability"></a>

## Function `cancel_burn_with_capability`

Cancels the burn request in the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a></code> resource stored at <code>preburn_address</code> and
return the coins to the caller.
This function can only be called by the holder of a
<code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>, and will fail if the <code><a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;</code> resource
at <code>preburn_address</code> does not contain a pending burn request.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;CoinType&gt;(preburn_address: address, _capability: &<a href="DiemTest.md#0x1_DiemTest_BurnCapability">DiemTest::BurnCapability</a>&lt;CoinType&gt;): <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;CoinType&gt;(
    preburn_address: address,
    _capability: &<a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
): <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt; <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>, <a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a> {
    // destroy the coin in the preburn area
    <b>let</b> preburn = borrow_global_mut&lt;<a href="DiemTest.md#0x1_DiemTest_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address);
    <b>let</b> coin = <a href="DiemTest.md#0x1_DiemTest_withdraw_all">withdraw_all</a>&lt;CoinType&gt;(&<b>mut</b> preburn.to_burn);
    // <b>update</b> the market cap
    <b>let</b> currency_code = <a href="DiemTest.md#0x1_DiemTest_currency_code">currency_code</a>&lt;CoinType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    <b>let</b> amount = <a href="DiemTest.md#0x1_DiemTest_value">value</a>(&coin);
    info.preburn_value = info.preburn_value - amount;
    // Don't emit cancel burn events for synthetic currencies. cancel burn shouldn't be be used
    // for synthetics in the first place
    <b>if</b> (!info.is_synthetic) {
        <a href="_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.cancel_burn_events,
            <a href="DiemTest.md#0x1_DiemTest_CancelBurnEvent">CancelBurnEvent</a> {
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

<a name="0x1_DiemTest_remove_burn_capability"></a>

## Function `remove_burn_capability`

Removes and returns the <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> from <code>account</code>.
Calls to this function will fail if <code>account</code> does  not have a
published <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> resource at the top-level.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_remove_burn_capability">remove_burn_capability</a>&lt;CoinType&gt;(account: &signer): <a href="DiemTest.md#0x1_DiemTest_BurnCapability">DiemTest::BurnCapability</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_remove_burn_capability">remove_burn_capability</a>&lt;CoinType&gt;(account: &signer): <a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a> {
    move_from&lt;<a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="_address_of">Signer::address_of</a>(account))
}
</code></pre>



</details>

<a name="0x1_DiemTest_preburn_value"></a>

## Function `preburn_value`

Returns the total value of <code><a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;</code> that is waiting to be
burned throughout the system (i.e. the sum of all outstanding
preburn requests across all preburn resources for the <code>CoinType</code>
currency).


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_preburn_value">preburn_value</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_preburn_value">preburn_value</a>&lt;CoinType&gt;(): u64 <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).preburn_value
}
</code></pre>



</details>

<a name="0x1_DiemTest_zero"></a>

## Function `zero`

Create a new <code><a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;</code> with a value of <code>0</code>. Anyone can call
this and it will be successful as long as <code>CoinType</code> is a registered currency.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_zero">zero</a>&lt;CoinType&gt;(): <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_zero">zero</a>&lt;CoinType&gt;(): <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt; {
    <a href="DiemTest.md#0x1_DiemTest_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt; { value: 0 }
}
</code></pre>



</details>

<a name="0x1_DiemTest_value"></a>

## Function `value`

Returns the <code>value</code> of the passed in <code>coin</code>. The value is
represented in the base units for the currency represented by
<code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_value">value</a>&lt;CoinType&gt;(coin: &<a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_value">value</a>&lt;CoinType&gt;(coin: &<a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;): u64 {
    coin.value
}
</code></pre>



</details>

<a name="0x1_DiemTest_split"></a>

## Function `split`

Removes <code>amount</code> of value from the passed in <code>coin</code>. Returns the
remaining balance of the passed in <code>coin</code>, along with another coin
with value equal to <code>amount</code>. Calls will fail if <code>amount &gt; Diem::value(&coin)</code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_split">split</a>&lt;CoinType&gt;(coin: <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;, amount: u64): (<a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;, <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_split">split</a>&lt;CoinType&gt;(coin: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;, amount: u64): (<a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;, <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;) {
    <b>let</b> other = <a href="DiemTest.md#0x1_DiemTest_withdraw">withdraw</a>(&<b>mut</b> coin, amount);
    (coin, other)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> coin.<a href="DiemTest.md#0x1_DiemTest_value">value</a> &lt; amount;
<b>ensures</b> result_1.value == coin.value - amount;
<b>ensures</b> result_2.value == amount;
</code></pre>



</details>

<a name="0x1_DiemTest_withdraw"></a>

## Function `withdraw`

Withdraw <code>amount</code> from the passed-in <code>coin</code>, where the original coin is modified in place.
After this function is executed, the original <code>coin</code> will have
<code>value = original_value - amount</code>, and the new coin will have a <code>value = amount</code>.
Calls will abort if the passed-in <code>amount</code> is greater than the
value of the passed-in <code>coin</code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_withdraw">withdraw</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;, amount: u64): <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_withdraw">withdraw</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;, amount: u64): <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt; {
    // Check that `amount` is less than the coin's value
    <b>assert</b>(coin.value &gt;= amount, <a href="DiemTest.md#0x1_DiemTest_EAMOUNT_EXCEEDS_COIN_VALUE">EAMOUNT_EXCEEDS_COIN_VALUE</a>);
    coin.value = coin.value - amount;
    <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a> { value: amount }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> coin.<a href="DiemTest.md#0x1_DiemTest_value">value</a> &lt; amount;
<b>ensures</b> coin.value == <b>old</b>(coin.value) - amount;
<b>ensures</b> result.value == amount;
</code></pre>



</details>

<a name="0x1_DiemTest_withdraw_all"></a>

## Function `withdraw_all`

Return a <code><a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;</code> worth <code>coin.value</code> and reduces the <code>value</code> of the input <code>coin</code> to
zero. Does not abort.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_withdraw_all">withdraw_all</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;): <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_withdraw_all">withdraw_all</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;): <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt; {
    <b>let</b> val = coin.value;
    <a href="DiemTest.md#0x1_DiemTest_withdraw">withdraw</a>(coin, val)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result.value == <b>old</b>(coin.value);
<b>ensures</b> coin.value == 0;
</code></pre>



</details>

<a name="0x1_DiemTest_join"></a>

## Function `join`

and returns a new coin whose value is equal to the sum of the two inputs.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_join">join</a>&lt;CoinType&gt;(xus: <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;, coin2: <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;): <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_join">join</a>&lt;CoinType&gt;(xus: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;, coin2: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;): <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;  {
    <a href="DiemTest.md#0x1_DiemTest_deposit">deposit</a>(&<b>mut</b> xus, coin2);
    xus
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> xus.value + coin2.value &gt; max_u64();
<b>ensures</b> result.value == xus.value + coin2.value;
</code></pre>



</details>

<a name="0x1_DiemTest_deposit"></a>

## Function `deposit`

"Merges" the two coins.
The coin passed in by reference will have a value equal to the sum of the two coins
The <code>check</code> coin is consumed in the process


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_deposit">deposit</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;, check: <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_deposit">deposit</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;, check: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a> { value } = check;
    coin.value = coin.value + value;
}
</code></pre>



</details>

<a name="0x1_DiemTest_destroy_zero"></a>

## Function `destroy_zero`

Destroy a zero-value coin. Calls will fail if the <code>value</code> in the passed-in <code>coin</code> is non-zero
so you cannot "burn" any non-zero amount of <code><a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a></code> without having
a <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a></code> for the specific <code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(coin: <a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(coin: <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a> { value } = coin;
    <b>assert</b>(value == 0, <a href="DiemTest.md#0x1_DiemTest_EDESTRUCTION_OF_NONZERO_COIN">EDESTRUCTION_OF_NONZERO_COIN</a>)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> coin.value &gt; 0;
</code></pre>



</details>

<a name="0x1_DiemTest_register_currency"></a>

## Function `register_currency`

Register the type <code>CoinType</code> as a currency. Until the type is
registered as a currency it cannot be used as a coin/currency unit in Diem.
The passed-in <code>dr_account</code> must be a specific address (<code><a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code>) and
<code>dr_account</code> must also have the correct <code><a href="DiemTest.md#0x1_DiemTest_RegisterNewCurrency">RegisterNewCurrency</a></code> capability.
After the first registration of <code>CoinType</code> as a
currency, additional attempts to register <code>CoinType</code> as a currency
will abort.
When the <code>CoinType</code> is registered it publishes the
<code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;</code> resource under the <code><a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code> and
adds the currency to the set of <code><a href="">RegisteredCurrencies</a></code>. It returns
<code><a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> and <code><a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> resources.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_register_currency">register_currency</a>&lt;CoinType&gt;(dr_account: &signer, to_xdx_exchange_rate: <a href="_FixedPoint32">FixedPoint32::FixedPoint32</a>, is_synthetic: bool, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;): (<a href="DiemTest.md#0x1_DiemTest_MintCapability">DiemTest::MintCapability</a>&lt;CoinType&gt;, <a href="DiemTest.md#0x1_DiemTest_BurnCapability">DiemTest::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_register_currency">register_currency</a>&lt;CoinType&gt;(
    dr_account: &signer,
    to_xdx_exchange_rate: <a href="">FixedPoint32</a>,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: vector&lt;u8&gt;,
): (<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;, <a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;)
{
    <a href="_assert_diem_root">Roles::assert_diem_root</a>(dr_account);
    // Operational constraint that it must be stored under a specific address.
    <b>assert</b>(
        <a href="_address_of">Signer::address_of</a>(dr_account) == <a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>(),
        <a href="DiemTest.md#0x1_DiemTest_EINVALID_SINGLETON_ADDRESS">EINVALID_SINGLETON_ADDRESS</a>
    );

    move_to(dr_account, <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt; {
        total_value: 0,
        preburn_value: 0,
        to_xdx_exchange_rate,
        is_synthetic,
        scaling_factor,
        fractional_part,
        currency_code: <b>copy</b> currency_code,
        can_mint: <b>true</b>,
        mint_events: <a href="_new_event_handle">Event::new_event_handle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_MintEvent">MintEvent</a>&gt;(dr_account),
        burn_events: <a href="_new_event_handle">Event::new_event_handle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_BurnEvent">BurnEvent</a>&gt;(dr_account),
        preburn_events: <a href="_new_event_handle">Event::new_event_handle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_PreburnEvent">PreburnEvent</a>&gt;(dr_account),
        cancel_burn_events: <a href="_new_event_handle">Event::new_event_handle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_CancelBurnEvent">CancelBurnEvent</a>&gt;(dr_account),
        exchange_rate_update_events: <a href="_new_event_handle">Event::new_event_handle</a>&lt;<a href="DiemTest.md#0x1_DiemTest_ToXDXExchangeRateUpdateEvent">ToXDXExchangeRateUpdateEvent</a>&gt;(dr_account)
    });
    <a href="_add_currency_code">RegisteredCurrencies::add_currency_code</a>(
        dr_account,
        currency_code,
    );
    (<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;{}, <a href="DiemTest.md#0x1_DiemTest_BurnCapability">BurnCapability</a>&lt;CoinType&gt;{})
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> !<a href="_spec_has_diem_root_role_addr">Roles::spec_has_diem_root_role_addr</a>(<a href="_spec_address_of">Signer::spec_address_of</a>(dr_account));
<b>aborts_if</b> <a href="_spec_address_of">Signer::spec_address_of</a>(dr_account) != <a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_spec_address_of">Signer::spec_address_of</a>(dr_account));
<b>aborts_if</b> <a href="DiemTest.md#0x1_DiemTest_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;();
<b>include</b> <a href="_AddCurrencyCodeAbortsIf">RegisteredCurrencies::AddCurrencyCodeAbortsIf</a>;
</code></pre>



</details>

<a name="0x1_DiemTest_register_SCS_currency"></a>

## Function `register_SCS_currency`

Registers a stable currency (SCS) coin -- i.e., a non-synthetic currency.
Resources are published on two distinct
accounts: The CoinInfo is published on the Diem root account, and the mint and
burn capabilities are published on a treasury compliance account.
This code allows different currencies to have different treasury compliance
accounts.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_register_SCS_currency">register_SCS_currency</a>&lt;CoinType&gt;(dr_account: &signer, tc_account: &signer, to_xdx_exchange_rate: <a href="_FixedPoint32">FixedPoint32::FixedPoint32</a>, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_register_SCS_currency">register_SCS_currency</a>&lt;CoinType&gt;(
    dr_account: &signer,
    tc_account: &signer,
    to_xdx_exchange_rate: <a href="">FixedPoint32</a>,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: vector&lt;u8&gt;,
) {
    <a href="_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>let</b> (mint_cap, burn_cap) =
        <a href="DiemTest.md#0x1_DiemTest_register_currency">register_currency</a>&lt;CoinType&gt;(
            dr_account,
            to_xdx_exchange_rate,
            <b>false</b>,   // is_synthetic
            scaling_factor,
            fractional_part,
            currency_code,
        );
    // DD: converted <b>to</b> move_to because of problems proving <b>invariant</b>.
    // publish_mint_capability&lt;CoinType&gt;(tc_account, mint_cap, tc_account);
    move_to(tc_account, mint_cap);
    <a href="DiemTest.md#0x1_DiemTest_publish_burn_capability">publish_burn_capability</a>&lt;CoinType&gt;(tc_account, burn_cap, tc_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
<b>ensures</b> <a href="DiemTest.md#0x1_DiemTest_spec_has_mint_capability">spec_has_mint_capability</a>&lt;CoinType&gt;(<a href="_spec_address_of">Signer::spec_address_of</a>(tc_account));
</code></pre>



</details>

<a name="0x1_DiemTest_market_cap"></a>

## Function `market_cap`

Returns the total amount of currency minted of type <code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_market_cap">market_cap</a>&lt;CoinType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_market_cap">market_cap</a>&lt;CoinType&gt;(): u128
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).total_value
}
</code></pre>



</details>

<a name="0x1_DiemTest_approx_xdx_for_value"></a>

## Function `approx_xdx_for_value`

Returns the value of the coin in the <code>FromCoinType</code> currency in XDX.
This should only be used where a _rough_ approximation of the exchange
rate is needed.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_approx_xdx_for_value">approx_xdx_for_value</a>&lt;FromCoinType&gt;(from_value: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_approx_xdx_for_value">approx_xdx_for_value</a>&lt;FromCoinType&gt;(from_value: u64): u64
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> xdx_exchange_rate = <a href="DiemTest.md#0x1_DiemTest_xdx_exchange_rate">xdx_exchange_rate</a>&lt;FromCoinType&gt;();
    <a href="_multiply_u64">FixedPoint32::multiply_u64</a>(from_value, xdx_exchange_rate)
}
</code></pre>



</details>

<a name="0x1_DiemTest_approx_xdx_for_coin"></a>

## Function `approx_xdx_for_coin`

Returns the value of the coin in the <code>FromCoinType</code> currency in XDX.
This should only be used where a rough approximation of the exchange
rate is needed.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_approx_xdx_for_coin">approx_xdx_for_coin</a>&lt;FromCoinType&gt;(coin: &<a href="DiemTest.md#0x1_DiemTest_Diem">DiemTest::Diem</a>&lt;FromCoinType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_approx_xdx_for_coin">approx_xdx_for_coin</a>&lt;FromCoinType&gt;(coin: &<a href="DiemTest.md#0x1_DiemTest_Diem">Diem</a>&lt;FromCoinType&gt;): u64
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> from_value = <a href="DiemTest.md#0x1_DiemTest_value">value</a>(coin);
    <a href="DiemTest.md#0x1_DiemTest_approx_xdx_for_value">approx_xdx_for_value</a>&lt;FromCoinType&gt;(from_value)
}
</code></pre>



</details>

<a name="0x1_DiemTest_is_currency"></a>

## Function `is_currency`

Returns <code><b>true</b></code> if the type <code>CoinType</code> is a registered currency.
Returns <code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_is_currency">is_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_is_currency">is_currency</a>&lt;CoinType&gt;(): bool {
    <b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_DiemTest_is_SCS_currency"></a>

## Function `is_SCS_currency`



<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_is_SCS_currency">is_SCS_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_is_SCS_currency">is_SCS_currency</a>&lt;CoinType&gt;(): bool <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> info = borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    !info.is_synthetic
}
</code></pre>



</details>

<a name="0x1_DiemTest_is_synthetic_currency"></a>

## Function `is_synthetic_currency`

Returns <code><b>true</b></code> if <code>CoinType</code> is a synthetic currency as defined in
its <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code>. Returns <code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(): bool
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> addr = <a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>();
    <b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(addr) &&
        borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(addr).is_synthetic
}
</code></pre>



</details>

<a name="0x1_DiemTest_scaling_factor"></a>

## Function `scaling_factor`

Returns the scaling factor for the <code>CoinType</code> currency as defined
in its <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_scaling_factor">scaling_factor</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_scaling_factor">scaling_factor</a>&lt;CoinType&gt;(): u64
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).scaling_factor
}
</code></pre>



</details>

<a name="0x1_DiemTest_fractional_part"></a>

## Function `fractional_part`

Returns the representable (i.e. real-world) fractional part for the
<code>CoinType</code> currency as defined in its <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_fractional_part">fractional_part</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_fractional_part">fractional_part</a>&lt;CoinType&gt;(): u64
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).fractional_part
}
</code></pre>



</details>

<a name="0x1_DiemTest_currency_code"></a>

## Function `currency_code`

Returns the currency code for the registered currency as defined in
its <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_currency_code">currency_code</a>&lt;CoinType&gt;(): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_currency_code">currency_code</a>&lt;CoinType&gt;(): vector&lt;u8&gt;
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    *&borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).currency_code
}
</code></pre>



</details>

<a name="0x1_DiemTest_update_xdx_exchange_rate"></a>

## Function `update_xdx_exchange_rate`

Updates the <code>to_xdx_exchange_rate</code> held in the <code><a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a></code> for
<code>FromCoinType</code> to the new passed-in <code>xdx_exchange_rate</code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_update_xdx_exchange_rate">update_xdx_exchange_rate</a>&lt;FromCoinType&gt;(tr_account: &signer, xdx_exchange_rate: <a href="_FixedPoint32">FixedPoint32::FixedPoint32</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_update_xdx_exchange_rate">update_xdx_exchange_rate</a>&lt;FromCoinType&gt;(
    tr_account: &signer,
    xdx_exchange_rate: <a href="">FixedPoint32</a>
) <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>assert</b>(<a href="_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(tr_account), <a href="DiemTest.md#0x1_DiemTest_ENOT_TREASURY_COMPLIANCE">ENOT_TREASURY_COMPLIANCE</a>);
    <a href="DiemTest.md#0x1_DiemTest_assert_is_currency">assert_is_currency</a>&lt;FromCoinType&gt;();
    <b>let</b> currency_info = borrow_global_mut&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;FromCoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    currency_info.to_xdx_exchange_rate = xdx_exchange_rate;
    <a href="_emit_event">Event::emit_event</a>(
        &<b>mut</b> currency_info.exchange_rate_update_events,
        <a href="DiemTest.md#0x1_DiemTest_ToXDXExchangeRateUpdateEvent">ToXDXExchangeRateUpdateEvent</a> {
            currency_code: *&currency_info.currency_code,
            new_to_xdx_exchange_rate: <a href="_get_raw_value">FixedPoint32::get_raw_value</a>(*&currency_info.to_xdx_exchange_rate),
        }
    );

}
</code></pre>



</details>

<a name="0x1_DiemTest_xdx_exchange_rate"></a>

## Function `xdx_exchange_rate`

Returns the (rough) exchange rate between <code>CoinType</code> and <code>XDX</code>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_xdx_exchange_rate">xdx_exchange_rate</a>&lt;CoinType&gt;(): <a href="_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_xdx_exchange_rate">xdx_exchange_rate</a>&lt;CoinType&gt;(): <a href="">FixedPoint32</a>
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    *&borrow_global&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).to_xdx_exchange_rate
}
</code></pre>



</details>

<a name="0x1_DiemTest_update_minting_ability"></a>

## Function `update_minting_ability`

There may be situations in which we disallow the further minting of
coins in the system without removing the currency. This function
allows the association TC account to control whether or not further coins of
<code>CoinType</code> can be minted or not. If this is called with <code>can_mint =
<b>true</b></code>, then minting is allowed, if <code>can_mint = <b>false</b></code> then minting is
disallowed until it is turned back on via this function. All coins
start out in the default state of <code>can_mint = <b>true</b></code>.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_update_minting_ability">update_minting_ability</a>&lt;CoinType&gt;(tr_account: &signer, can_mint: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTest.md#0x1_DiemTest_update_minting_ability">update_minting_ability</a>&lt;CoinType&gt;(
    tr_account: &signer,
    can_mint: bool,
    )
<b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>assert</b>(<a href="_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(tr_account), <a href="DiemTest.md#0x1_DiemTest_ENOT_TREASURY_COMPLIANCE">ENOT_TREASURY_COMPLIANCE</a>);
    <a href="DiemTest.md#0x1_DiemTest_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;();
    <b>let</b> currency_info = borrow_global_mut&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    currency_info.can_mint = can_mint;
}
</code></pre>



</details>

<a name="0x1_DiemTest_assert_is_currency"></a>

## Function `assert_is_currency`

Asserts that <code>CoinType</code> is a registered currency.


<pre><code><b>fun</b> <a href="DiemTest.md#0x1_DiemTest_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemTest.md#0x1_DiemTest_assert_is_currency">assert_is_currency</a>&lt;CoinType&gt;() {
    <b>assert</b>(<a href="DiemTest.md#0x1_DiemTest_is_currency">is_currency</a>&lt;CoinType&gt;(), <a href="DiemTest.md#0x1_DiemTest_ENOT_A_REGISTERED_CURRENCY">ENOT_A_REGISTERED_CURRENCY</a>);
}
</code></pre>



</details>

<a name="0x1_DiemTest_assert_is_SCS_currency"></a>

## Function `assert_is_SCS_currency`



<pre><code><b>fun</b> <a href="DiemTest.md#0x1_DiemTest_assert_is_SCS_currency">assert_is_SCS_currency</a>&lt;CoinType&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemTest.md#0x1_DiemTest_assert_is_SCS_currency">assert_is_SCS_currency</a>&lt;CoinType&gt;() <b>acquires</b> <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a> {
    <b>assert</b>(<a href="DiemTest.md#0x1_DiemTest_is_SCS_currency">is_SCS_currency</a>&lt;CoinType&gt;(), <a href="DiemTest.md#0x1_DiemTest_ENOT_AN_SCS_CURRENCY">ENOT_AN_SCS_CURRENCY</a>);
}
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification

**************** MODULE SPECIFICATION ****************


<a name="@Module_Specification_2"></a>

### Module Specification


Verify all functions in this module.
> TODO(wrwg): temporarily deactivated as a recent PR destroyed assumptions
> about coin balance.


<pre><code><b>pragma</b> verify = <b>true</b>;
</code></pre>



Checks whether currency is registered. Mirrors <code><a href="DiemTest.md#0x1_DiemTest_is_currency">Self::is_currency</a>&lt;CoinType&gt;</code>.


<a name="0x1_DiemTest_spec_is_currency"></a>


<pre><code><b>define</b> <a href="DiemTest.md#0x1_DiemTest_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;(): bool {
    <b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>


Returns currency information.


<a name="0x1_DiemTest_spec_currency_info"></a>


<pre><code><b>define</b> <a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;(): <a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt; {
    <b>global</b>&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>


Specification version of <code><a href="DiemTest.md#0x1_DiemTest_approx_xdx_for_value">Self::approx_xdx_for_value</a></code>.


<a name="0x1_DiemTest_spec_approx_xdx_for_value"></a>


<pre><code><b>define</b> <a href="DiemTest.md#0x1_DiemTest_spec_approx_xdx_for_value">spec_approx_xdx_for_value</a>&lt;CoinType&gt;(value: num):  num {
    <a href="_spec_multiply_u64">FixedPoint32::spec_multiply_u64</a>(
        value,
        <b>global</b>&lt;<a href="DiemTest.md#0x1_DiemTest_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).to_xdx_exchange_rate
    )
}
<a name="0x1_DiemTest_spec_is_SCS_currency"></a>
<b>define</b> <a href="DiemTest.md#0x1_DiemTest_spec_is_SCS_currency">spec_is_SCS_currency</a>&lt;CoinType&gt;(): bool {
    <a href="DiemTest.md#0x1_DiemTest_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;() && !<a href="DiemTest.md#0x1_DiemTest_spec_currency_info">spec_currency_info</a>&lt;CoinType&gt;().is_synthetic
}
</code></pre>


Checks whether the currency has a mint capability.  This is only relevant for
SCS coins


<a name="0x1_DiemTest_spec_has_mint_capability"></a>


<pre><code><b>define</b> <a href="DiemTest.md#0x1_DiemTest_spec_has_mint_capability">spec_has_mint_capability</a>&lt;CoinType&gt;(addr1: address): bool {
    <b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(addr1)
}
</code></pre>



<a name="@Minting_3"></a>

#### Minting

For an SCS coin, the mint capability cannot move or disappear.
TODO: Specify that they're published at the one true treasurycompliance address?


<a name="0x1_DiemTest_MintCapabilitySpecs"></a>

If an address has a mint capability, it is an SCS currency.


<pre><code><b>schema</b> <a href="DiemTest.md#0x1_DiemTest_MintCapabilitySpecs">MintCapabilitySpecs</a> {
    <b>invariant</b> <b>module</b> <b>forall</b> coin_type: type
                         <b>where</b> (<b>exists</b> addr3: address : <a href="DiemTest.md#0x1_DiemTest_spec_has_mint_capability">spec_has_mint_capability</a>&lt;coin_type&gt;(addr3)) :
                              <a href="DiemTest.md#0x1_DiemTest_spec_is_SCS_currency">spec_is_SCS_currency</a>&lt;coin_type&gt;();
}
</code></pre>


If there is a pending offer for a mint capability, the coin_type is an SCS currency and
there are no published Mint Capabilities. (This is the state after register_SCS_currency_start)


<pre><code><b>schema</b> <a href="DiemTest.md#0x1_DiemTest_MintCapabilitySpecs">MintCapabilitySpecs</a> {
    <b>invariant</b> <b>module</b> <b>forall</b> coin_type: type :
                              <a href="DiemTest.md#0x1_DiemTest_spec_is_SCS_currency">spec_is_SCS_currency</a>&lt;coin_type&gt;()
                              && (<b>forall</b> addr3: address : !<a href="DiemTest.md#0x1_DiemTest_spec_has_mint_capability">spec_has_mint_capability</a>&lt;coin_type&gt;(addr3));
    <b>invariant</b> <b>module</b> <b>forall</b> coin_type: type <b>where</b> <a href="DiemTest.md#0x1_DiemTest_spec_is_SCS_currency">spec_is_SCS_currency</a>&lt;coin_type&gt;():
        <b>forall</b> addr1: address, addr2: address
             <b>where</b> <b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;coin_type&gt;&gt;(addr1) && <b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;coin_type&gt;&gt;(addr2):
                  addr1 == addr2;
    <b>ensures</b> <b>forall</b> coin_type: type:
        <b>forall</b> addr1: address <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;coin_type&gt;&gt;(addr1)):
            <b>exists</b>&lt;<a href="DiemTest.md#0x1_DiemTest_MintCapability">MintCapability</a>&lt;coin_type&gt;&gt;(addr1);
}
</code></pre>




<pre><code><b>apply</b> <a href="DiemTest.md#0x1_DiemTest_MintCapabilitySpecs">MintCapabilitySpecs</a> <b>to</b> *&lt;T&gt;, *;
</code></pre>



<a name="@Conservation_of_currency_4"></a>

#### Conservation of currency


Maintain a spec variable representing the sum of
all coins of a currency type.


<a name="0x1_DiemTest_sum_of_coin_values"></a>


<pre><code><b>global</b> <a href="DiemTest.md#0x1_DiemTest_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt;: num;
</code></pre>
