
<a name="0x0_Libra"></a>

# Module `0x0::Libra`

### Table of Contents

-  [Struct `Libra`](#0x0_Libra_Libra)
-  [Struct `MintCapability`](#0x0_Libra_MintCapability)
-  [Struct `BurnCapability`](#0x0_Libra_BurnCapability)
-  [Struct `CurrencyRegistrationCapability`](#0x0_Libra_CurrencyRegistrationCapability)
-  [Struct `MintEvent`](#0x0_Libra_MintEvent)
-  [Struct `BurnEvent`](#0x0_Libra_BurnEvent)
-  [Struct `PreburnEvent`](#0x0_Libra_PreburnEvent)
-  [Struct `CancelBurnEvent`](#0x0_Libra_CancelBurnEvent)
-  [Struct `CurrencyInfo`](#0x0_Libra_CurrencyInfo)
-  [Struct `Preburn`](#0x0_Libra_Preburn)
-  [Struct `AddCurrency`](#0x0_Libra_AddCurrency)
-  [Function `initialize`](#0x0_Libra_initialize)
-  [Function `grant_mint_capability_to_association`](#0x0_Libra_grant_mint_capability_to_association)
-  [Function `publish_mint_capability`](#0x0_Libra_publish_mint_capability)
-  [Function `publish_burn_capability`](#0x0_Libra_publish_burn_capability)
-  [Function `mint`](#0x0_Libra_mint)
-  [Function `burn`](#0x0_Libra_burn)
-  [Function `cancel_burn`](#0x0_Libra_cancel_burn)
-  [Function `new_preburn`](#0x0_Libra_new_preburn)
-  [Function `mint_with_capability`](#0x0_Libra_mint_with_capability)
-  [Function `preburn_with_resource`](#0x0_Libra_preburn_with_resource)
-  [Function `create_preburn`](#0x0_Libra_create_preburn)
-  [Function `publish_preburn_to_account`](#0x0_Libra_publish_preburn_to_account)
-  [Function `preburn_to`](#0x0_Libra_preburn_to)
-  [Function `burn_with_capability`](#0x0_Libra_burn_with_capability)
-  [Function `burn_with_resource_cap`](#0x0_Libra_burn_with_resource_cap)
-  [Function `cancel_burn_with_capability`](#0x0_Libra_cancel_burn_with_capability)
-  [Function `remove_mint_capability`](#0x0_Libra_remove_mint_capability)
-  [Function `remove_burn_capability`](#0x0_Libra_remove_burn_capability)
-  [Function `preburn_value`](#0x0_Libra_preburn_value)
-  [Function `zero`](#0x0_Libra_zero)
-  [Function `value`](#0x0_Libra_value)
-  [Function `split`](#0x0_Libra_split)
-  [Function `withdraw`](#0x0_Libra_withdraw)
-  [Function `join`](#0x0_Libra_join)
-  [Function `deposit`](#0x0_Libra_deposit)
-  [Function `destroy_zero`](#0x0_Libra_destroy_zero)
-  [Function `register_currency`](#0x0_Libra_register_currency)
-  [Function `market_cap`](#0x0_Libra_market_cap)
-  [Function `approx_lbr_for_value`](#0x0_Libra_approx_lbr_for_value)
-  [Function `approx_lbr_for_coin`](#0x0_Libra_approx_lbr_for_coin)
-  [Function `is_currency`](#0x0_Libra_is_currency)
-  [Function `is_synthetic_currency`](#0x0_Libra_is_synthetic_currency)
-  [Function `scaling_factor`](#0x0_Libra_scaling_factor)
-  [Function `fractional_part`](#0x0_Libra_fractional_part)
-  [Function `currency_code`](#0x0_Libra_currency_code)
-  [Function `update_lbr_exchange_rate`](#0x0_Libra_update_lbr_exchange_rate)
-  [Function `lbr_exchange_rate`](#0x0_Libra_lbr_exchange_rate)
-  [Function `update_minting_ability`](#0x0_Libra_update_minting_ability)
-  [Function `assert_assoc_and_currency`](#0x0_Libra_assert_assoc_and_currency)
-  [Function `assert_is_coin`](#0x0_Libra_assert_is_coin)
-  [Specification](#0x0_Libra_Specification)
    -  [Struct `Libra`](#0x0_Libra_Specification_Libra)
    -  [Function `register_currency`](#0x0_Libra_Specification_register_currency)
    -  [Function `is_currency`](#0x0_Libra_Specification_is_currency)
    -  [Function `assert_is_coin`](#0x0_Libra_Specification_assert_is_coin)



<a name="0x0_Libra_Libra"></a>

## Struct `Libra`

The
<code><a href="#0x0_Libra">Libra</a></code> resource defines the Libra coin for each currency in
Libra. Each "coin" is coupled with a type
<code>CoinType</code> specifying the
currency of the coin, and a
<code>value</code> field specifying the value
of the coin (in the base units of the currency
<code>CoinType</code>
and specified in the
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code> resource for that
<code>CoinType</code>
published under the
<code><a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code> account address).


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;
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

<a name="0x0_Libra_MintCapability"></a>

## Struct `MintCapability`

The
<code><a href="#0x0_Libra_MintCapability">MintCapability</a></code> resource defines a capability to allow minting
of coins of
<code>CoinType</code> currency by the holder of this capability.
This capability is held only either by the CoreAddresses::TREASURY_COMPLIANCE_ADDRESS() account or the
<code><a href="LBR.md#0x0_LBR">0x0::LBR</a></code> module (and
<code><a href="CoreAddresses.md#0x0_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>()</code> in testnet).


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;
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

<a name="0x0_Libra_BurnCapability"></a>

## Struct `BurnCapability`

The
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a></code> resource defines a capability to allow coins
of
<code>CoinType</code> currency to be burned by the holder of the
capability. This capability is only held by the
<code><a href="CoreAddresses.md#0x0_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()</code> account,
and the
<code><a href="LBR.md#0x0_LBR">0x0::LBR</a></code> module (and
<code><a href="CoreAddresses.md#0x0_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>()</code> in testnet).


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
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

<a name="0x0_Libra_CurrencyRegistrationCapability"></a>

## Struct `CurrencyRegistrationCapability`

The
<code><a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a></code> is a singleton resource
published under the
<code><a href="CoreAddresses.md#0x0_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>()</code> and grants
the capability to the
<code><a href="#0x0_Libra">0x0::Libra</a></code> module to add currencies to the
<code><a href="RegisteredCurrencies.md#0x0_RegisteredCurrencies">0x0::RegisteredCurrencies</a></code> on-chain config.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="RegisteredCurrencies.md#0x0_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a></code>
</dt>
<dd>
 A capability to allow updating the set of registered currencies on-chain.
</dd>
</dl>


</details>

<a name="0x0_Libra_MintEvent"></a>

## Struct `MintEvent`

A
<code><a href="#0x0_Libra_MintEvent">MintEvent</a></code> is emitted every time a Libra coin is minted. This
contains the
<code>amount</code> minted (in base units of the currency being
minted) along with the
<code>currency_code</code> for the coin(s) being
minted, and that is defined in the
<code>currency_code</code> field of the
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code> resource for the currency.


<pre><code><b>struct</b> <a href="#0x0_Libra_MintEvent">MintEvent</a>
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

<a name="0x0_Libra_BurnEvent"></a>

## Struct `BurnEvent`

A
<code><a href="#0x0_Libra_BurnEvent">BurnEvent</a></code> is emitted every time a non-synthetic[1] Libra coin is
burned. It contains the
<code>amount</code> burned in base units for the
currency, along with the
<code>currency_code</code> for the coins being burned
(and as defined in the
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code> resource for that currency).
It also contains the
<code>preburn_address</code> from which the coin is
extracted for burning.
[1] As defined by the
<code>is_synthetic</code> field in the
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code>
for that currency.


<pre><code><b>struct</b> <a href="#0x0_Libra_BurnEvent">BurnEvent</a>
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
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource that stored the now-burned funds
</dd>
</dl>


</details>

<a name="0x0_Libra_PreburnEvent"></a>

## Struct `PreburnEvent`

A
<code><a href="#0x0_Libra_PreburnEvent">PreburnEvent</a></code> is emitted every time an
<code>amount</code> of funds with
a coin type
<code>currency_code</code> are moved to a
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource under
the account at the address
<code>preburn_address</code>.


<pre><code><b>struct</b> <a href="#0x0_Libra_PreburnEvent">PreburnEvent</a>
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
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource that now holds the funds
</dd>
</dl>


</details>

<a name="0x0_Libra_CancelBurnEvent"></a>

## Struct `CancelBurnEvent`

A
<code><a href="#0x0_Libra_CancelBurnEvent">CancelBurnEvent</a></code> is emitted every time funds of
<code>amount</code> in a
<code><a href="#0x0_Libra_Preburn">Preburn</a></code>
resource at
<code>preburn_address</code> is canceled (removed from the
preburn, but not burned). The currency of the funds is given by the
<code>currency_code</code> as defined in the
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code> for that currency.


<pre><code><b>struct</b> <a href="#0x0_Libra_CancelBurnEvent">CancelBurnEvent</a>
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
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource that held the now-returned funds.
</dd>
</dl>


</details>

<a name="0x0_Libra_CurrencyInfo"></a>

## Struct `CurrencyInfo`

The
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;</code> resource stores the various
pieces of information needed for a currency (
<code>CoinType</code>) that is
registered on-chain. This resource _must_ be published under the
address given by
<code><a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code> in order for the registration of
<code>CoinType</code> as a recognized currency on-chain to be successful. At
the time of registration the
<code><a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> and
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> capabilities are returned to the caller.
Unless they are specified otherwise the fields in this resource are immutable.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;
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

<code>to_lbr_exchange_rate: <a href="FixedPoint32.md#0x0_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a></code>
</dt>
<dd>
 The (rough) exchange rate from
<code>CoinType</code> to
<code><a href="LBR.md#0x0_LBR">LBR</a></code>. Mutable.
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
<code><a href="Coin1.md#0x0_Coin1">Coin1</a></code>
</dd>
<dt>

<code>fractional_part: u64</code>
</dt>
<dd>
 The smallest fractional part (number of decimal places) to be
 used in the human-readable representation for the currency (e.g.
 10^2 for
<code><a href="Coin1.md#0x0_Coin1">Coin1</a></code> cents)
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

<code>mint_events: <a href="Event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_Libra_MintEvent">Libra::MintEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for minting and where
<code><a href="#0x0_Libra_MintEvent">MintEvent</a></code>s will be emitted.
</dd>
<dt>

<code>burn_events: <a href="Event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_Libra_BurnEvent">Libra::BurnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for burning, and where
<code><a href="#0x0_Libra_BurnEvent">BurnEvent</a></code>s will be emitted.
</dd>
<dt>

<code>preburn_events: <a href="Event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_Libra_PreburnEvent">Libra::PreburnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for preburn requests, and where all
 <code><a href="#0x0_Libra_PreburnEvent">PreburnEvent</a></code>s for this
<code>CoinType</code> will be emitted.
</dd>
<dt>

<code>cancel_burn_events: <a href="Event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_Libra_CancelBurnEvent">Libra::CancelBurnEvent</a>&gt;</code>
</dt>
<dd>
 Event stream for all cancelled preburn requests for this
 <code>CoinType</code>.
</dd>
</dl>


</details>

<a name="0x0_Libra_Preburn"></a>

## Struct `Preburn`

A holding area where funds that will subsequently be burned wait while their underlying
assets are moved off-chain.
This resource can only be created by the holder of a
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a></code>. An account that
contains this address has the authority to initiate a burn request. A burn request can be
resolved by the holder of a
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a></code> by either (1) burning the funds, or (2)
returning the funds to the account that initiated the burn request.
This design supports multiple preburn requests in flight at the same time,
including multiple burn requests from the same account. However, burn requests
(and cancellations) from the same account must be resolved in FIFO order.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>requests: vector&lt;<a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;&gt;</code>
</dt>
<dd>
 The queue of pending burn requests
</dd>
</dl>


</details>

<a name="0x0_Libra_AddCurrency"></a>

## Struct `AddCurrency`

An association account holding this privilege can add/remove the
currencies from the system. This must be published under the
address at
<code><a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code>.


<pre><code><b>struct</b> <a href="#0x0_Libra_AddCurrency">AddCurrency</a>
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

<a name="0x0_Libra_initialize"></a>

## Function `initialize`

Initialization of the
<code><a href="#0x0_Libra">Libra</a></code> module; initializes the set of
registered currencies in the
<code><a href="RegisteredCurrencies.md#0x0_RegisteredCurrencies">0x0::RegisteredCurrencies</a></code> on-chain
config, and publishes the
<code><a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a></code> under the
<code><a href="CoreAddresses.md#0x0_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>()</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_initialize">initialize</a>(config_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_initialize">initialize</a>(config_account: &signer) {
    Transaction::assert(
        <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(config_account) == <a href="CoreAddresses.md#0x0_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>(),
        0
    );
    <b>let</b> cap = <a href="RegisteredCurrencies.md#0x0_RegisteredCurrencies_initialize">RegisteredCurrencies::initialize</a>(config_account);
    move_to(config_account, <a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a>{ cap })
}
</code></pre>



</details>

<a name="0x0_Libra_grant_mint_capability_to_association"></a>

## Function `grant_mint_capability_to_association`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_mint_capability_to_association">grant_mint_capability_to_association</a>&lt;CoinType&gt;(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_mint_capability_to_association">grant_mint_capability_to_association</a>&lt;CoinType&gt;(association: &signer) {
    <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;(association);
    move_to(association, <a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;{})
}
</code></pre>



</details>

<a name="0x0_Libra_publish_mint_capability"></a>

## Function `publish_mint_capability`

Publishes the
<code><a href="#0x0_Libra_MintCapability">MintCapability</a></code>
<code>cap</code> for the
<code>CoinType</code> currency
under
<code>account</code>.
<code>CoinType</code>  must be a registered currency type,
and the
<code>account</code> must be an association account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_mint_capability">publish_mint_capability</a>&lt;CoinType&gt;(account: &signer, cap: <a href="#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_mint_capability">publish_mint_capability</a>&lt;CoinType&gt;(account: &signer, cap: <a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;) {
    <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;(account);
    move_to(account, cap)
}
</code></pre>



</details>

<a name="0x0_Libra_publish_burn_capability"></a>

## Function `publish_burn_capability`

Publishes the
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a></code>
<code>cap</code> for the
<code>CoinType</code> currency under
<code>account</code>.
<code>CoinType</code>
must be a registered currency type, and the
<code>account</code> must be an
association account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_burn_capability">publish_burn_capability</a>&lt;CoinType&gt;(account: &signer, cap: <a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_burn_capability">publish_burn_capability</a>&lt;CoinType&gt;(account: &signer, cap: <a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;) {
    <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;(account);
    move_to(account, cap)
}
</code></pre>



</details>

<a name="0x0_Libra_mint"></a>

## Function `mint`

Mints
<code>amount</code> coins. The
<code>account</code> must hold a
<code><a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> at the top-level in order for this call
to be successful, and will fail with
<code>MISSING_DATA</code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_mint">mint</a>&lt;CoinType&gt;(account: &signer, amount: u64): <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_mint">mint</a>&lt;CoinType&gt;(account: &signer, amount: u64): <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_MintCapability">MintCapability</a> {
    <a href="#0x0_Libra_mint_with_capability">mint_with_capability</a>(
        amount,
        borrow_global&lt;<a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account))
    )
}
</code></pre>



</details>

<a name="0x0_Libra_burn"></a>

## Function `burn`

Burns the coins currently held in the
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource held under
<code>preburn_address</code>.
Calls to this functions will fail if the
<code>account</code> does not have a
published
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a></code> for the
<code>CoinType</code> published under it.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn">burn</a>&lt;CoinType&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn">burn</a>&lt;CoinType&gt;(
    account: &signer,
    preburn_address: address
) <b>acquires</b> <a href="#0x0_Libra_BurnCapability">BurnCapability</a>, <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    <a href="#0x0_Libra_burn_with_capability">burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account))
    )
}
</code></pre>



</details>

<a name="0x0_Libra_cancel_burn"></a>

## Function `cancel_burn`

Cancels the oldest burn request in the
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource held
under the
<code>preburn_address</code>, and returns the coins.
Calls to this will fail if the sender does not have a published
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>, or if there are no preburn requests
outstanding in the
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource under
<code>preburn_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_cancel_burn">cancel_burn</a>&lt;CoinType&gt;(account: &signer, preburn_address: address): <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_cancel_burn">cancel_burn</a>&lt;CoinType&gt;(
    account: &signer,
    preburn_address: address
): <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt; <b>acquires</b> <a href="#0x0_Libra_BurnCapability">BurnCapability</a>, <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    <a href="#0x0_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account))
    )
}
</code></pre>



</details>

<a name="0x0_Libra_new_preburn"></a>

## Function `new_preburn`

Create a new
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource, and return it back to the sender.
The
<code>CoinType</code> must be a registered currency on-chain.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_new_preburn">new_preburn</a>&lt;CoinType&gt;(): <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_new_preburn">new_preburn</a>&lt;CoinType&gt;(): <a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt; {
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;();
    <a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt; { requests: <a href="Vector.md#0x0_Vector_empty">Vector::empty</a>() }
}
</code></pre>



</details>

<a name="0x0_Libra_mint_with_capability"></a>

## Function `mint_with_capability`

Mint a new
<code><a href="#0x0_Libra">Libra</a></code> coin of
<code>CoinType</code> currency worth
<code>value</code>. The
caller must have a reference to a
<code><a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code>. Only
the Association account or the
<code><a href="LBR.md#0x0_LBR">0x0::LBR</a></code> module can acquire such a
reference.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_mint_with_capability">mint_with_capability</a>&lt;CoinType&gt;(value: u64, _capability: &<a href="#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;): <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_mint_with_capability">mint_with_capability</a>&lt;CoinType&gt;(
    value: u64,
    _capability: &<a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;
): <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt; <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;();
    // TODO: temporary measure for testnet only: limit minting <b>to</b> 1B <a href="#0x0_Libra">Libra</a> at a time.
    // this is <b>to</b> prevent the market cap's total value from hitting u64_max due <b>to</b> excessive
    // minting. This will not be a problem in the production <a href="#0x0_Libra">Libra</a> system because coins will
    // be backed with real-world assets, and thus minting will be correspondingly rarer.
    // * 1000000 here because the unit is microlibra
    Transaction::assert(<a href="#0x0_Libra_value">value</a> &lt;= 1000000000 * 1000000, 11);
    <b>let</b> currency_code = <a href="#0x0_Libra_currency_code">currency_code</a>&lt;CoinType&gt;();
    // <b>update</b> market cap <b>resource</b> <b>to</b> reflect minting
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    Transaction::assert(info.can_mint, 4);
    info.total_value = info.total_value + (value <b>as</b> u128);
    // don't emit mint events for synthetic currenices
    <b>if</b> (!info.is_synthetic) {
        <a href="Event.md#0x0_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.mint_events,
            <a href="#0x0_Libra_MintEvent">MintEvent</a>{
                amount: value,
                currency_code,
            }
        );
    };

    <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt; { value }
}
</code></pre>



</details>

<a name="0x0_Libra_preburn_with_resource"></a>

## Function `preburn_with_resource`

Add the
<code>coin</code> to the
<code>preburn</code> queue in the
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource
held at the address
<code>preburn_address</code>. Emits a
<code><a href="#0x0_Libra_PreburnEvent">PreburnEvent</a></code> to
the
<code>preburn_events</code> event stream in the
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code> for the
<code>CoinType</code> passed in. However, if the currency being preburned is
<code>synthetic</code> then no
<code><a href="#0x0_Libra_PreburnEvent">PreburnEvent</a></code> event will be emitted.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_with_resource">preburn_with_resource</a>&lt;CoinType&gt;(coin: <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, preburn: &<b>mut</b> <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_with_resource">preburn_with_resource</a>&lt;CoinType&gt;(
    coin: <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;,
    preburn: &<b>mut</b> <a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;,
    preburn_address: address,
) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> coin_value = <a href="#0x0_Libra_value">value</a>(&coin);
    <a href="Vector.md#0x0_Vector_push_back">Vector::push_back</a>(
        &<b>mut</b> preburn.requests,
        coin
    );
    <b>let</b> currency_code = <a href="#0x0_Libra_currency_code">currency_code</a>&lt;CoinType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    info.preburn_value = info.preburn_value + coin_value;
    // don't emit preburn events for synthetic currencies
    <b>if</b> (!info.is_synthetic) {
        <a href="Event.md#0x0_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.preburn_events,
            <a href="#0x0_Libra_PreburnEvent">PreburnEvent</a>{
                amount: coin_value,
                currency_code,
                preburn_address,
            }
        );
    };
}
</code></pre>



</details>

<a name="0x0_Libra_create_preburn"></a>

## Function `create_preburn`

Create a
<code><a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code> resource


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_create_preburn">create_preburn</a>&lt;CoinType&gt;(creator: &signer): <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_create_preburn">create_preburn</a>&lt;CoinType&gt;(creator: &signer): <a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt; {
    // TODO: this should check for AssocRoot in the future
    <a href="Association.md#0x0_Association_assert_is_association">Association::assert_is_association</a>(creator);
    Transaction::assert(<a href="#0x0_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(), 201);
    <a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt; { requests: <a href="Vector.md#0x0_Vector_empty">Vector::empty</a>() }
}
</code></pre>



</details>

<a name="0x0_Libra_publish_preburn_to_account"></a>

## Function `publish_preburn_to_account`

Publishes a
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource under
<code>account</code>. This function is
used for bootstrapping the designated dealer at account-creation
time, and the association TC account
<code>creator</code> (at
<code><a href="CoreAddresses.md#0x0_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()</code>) is creating
this resource for the designated dealer.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_preburn_to_account">publish_preburn_to_account</a>&lt;CoinType&gt;(creator: &signer, account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_preburn_to_account">publish_preburn_to_account</a>&lt;CoinType&gt;(
    creator: &signer, account: &signer
) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    Transaction::assert(!<a href="#0x0_Libra_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(), 202);
    move_to(account, <a href="#0x0_Libra_create_preburn">create_preburn</a>&lt;CoinType&gt;(creator))
}
</code></pre>



</details>

<a name="0x0_Libra_preburn_to"></a>

## Function `preburn_to`

Sends
<code>coin</code> to the preburn queue for
<code>account</code>, where it will wait to either be burned
or returned to the balance of
<code>account</code>.
Calls to this function will fail if
<code>account</code> does not have a
<code><a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code> resource published under it.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_to">preburn_to</a>&lt;CoinType&gt;(account: &signer, coin: <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_to">preburn_to</a>&lt;CoinType&gt;(
    account: &signer, coin: <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    <b>let</b> sender = <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account);
    <a href="#0x0_Libra_preburn_with_resource">preburn_with_resource</a>(coin, borrow_global_mut&lt;<a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(sender), sender);
}
</code></pre>



</details>

<a name="0x0_Libra_burn_with_capability"></a>

## Function `burn_with_capability`

Permanently removes the coins held in the
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource stored at
<code>preburn_address</code> and
updates the market cap accordingly. If there are multiple preburn
requests in progress (i.e. in the preburn queue), this will remove the oldest one.
This function can only be called by the holder of a
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>.
Calls to this function will fail if the there is no
<code><a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code>
resource under
<code>preburn_address</code>, or, if the preburn queue for
<code>CoinType</code> has no pending burn requests.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn_with_capability">burn_with_capability</a>&lt;CoinType&gt;(preburn_address: address, capability: &<a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn_with_capability">burn_with_capability</a>&lt;CoinType&gt;(
    preburn_address: address,
    capability: &<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    // destroy the coin at the head of the preburn queue
    <a href="#0x0_Libra_burn_with_resource_cap">burn_with_resource_cap</a>(
        borrow_global_mut&lt;<a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address),
        preburn_address,
        capability
    )
}
</code></pre>



</details>

<a name="0x0_Libra_burn_with_resource_cap"></a>

## Function `burn_with_resource_cap`

Permanently removes the coins held in the
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource
<code>preburn</code> stored at
<code>preburn_address</code> and
updates the market cap accordingly. If there are multiple preburn
requests in progress (i.e. in the preburn queue), this will remove the oldest one.
This function can only be called by the holder of a
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>.
Calls to this function will fail if the there is no
<code><a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code>
resource under
<code>preburn_address</code>, or, if the preburn queue for
<code>CoinType</code> has no pending burn requests.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn_with_resource_cap">burn_with_resource_cap</a>&lt;CoinType&gt;(preburn: &<b>mut</b> <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;, preburn_address: address, _capability: &<a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn_with_resource_cap">burn_with_resource_cap</a>&lt;CoinType&gt;(
    preburn: &<b>mut</b> <a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;,
    preburn_address: address,
    _capability: &<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    // destroy the coin at the head of the preburn queue
    <b>let</b> <a href="#0x0_Libra">Libra</a> { value } = <a href="Vector.md#0x0_Vector_remove">Vector::remove</a>(&<b>mut</b> preburn.requests, 0);
    // <b>update</b> the market cap
    <b>let</b> currency_code = <a href="#0x0_Libra_currency_code">currency_code</a>&lt;CoinType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    info.total_value = info.total_value - (value <b>as</b> u128);
    info.preburn_value = info.preburn_value - value;
    // don't emit burn events for synthetic currencies
    <b>if</b> (!info.is_synthetic) {
        <a href="Event.md#0x0_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.burn_events,
            <a href="#0x0_Libra_BurnEvent">BurnEvent</a> {
                amount: value,
                currency_code,
                preburn_address,
            }
        );
    };
}
</code></pre>



</details>

<a name="0x0_Libra_cancel_burn_with_capability"></a>

## Function `cancel_burn_with_capability`

Cancels the burn request in the
<code><a href="#0x0_Libra_Preburn">Preburn</a></code> resource stored at
<code>preburn_address</code> and
return the coins to the caller.
If there are multiple preburn requests in progress for
<code>CoinType</code> (i.e. in the
preburn queue), this will cancel the oldest one.
This function can only be called by the holder of a
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code>, and will fail if the
<code><a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;</code> resource
at
<code>preburn_address</code> does not contain any pending burn requests.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;CoinType&gt;(preburn_address: address, _capability: &<a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;): <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;CoinType&gt;(
    preburn_address: address,
    _capability: &<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
): <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt; <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    // destroy the coin at the head of the preburn queue
    <b>let</b> preburn = borrow_global_mut&lt;<a href="#0x0_Libra_Preburn">Preburn</a>&lt;CoinType&gt;&gt;(preburn_address);
    <b>let</b> coin = <a href="Vector.md#0x0_Vector_remove">Vector::remove</a>(&<b>mut</b> preburn.requests, 0);
    // <b>update</b> the market cap
    <b>let</b> currency_code = <a href="#0x0_Libra_currency_code">currency_code</a>&lt;CoinType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    <b>let</b> amount = <a href="#0x0_Libra_value">value</a>(&coin);
    info.preburn_value = info.preburn_value - amount;
    // Don't emit cancel burn events for synthetic currencies. cancel burn shouldn't be be used
    // for synthetics in the first place
    <b>if</b> (!info.is_synthetic) {
        <a href="Event.md#0x0_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.cancel_burn_events,
            <a href="#0x0_Libra_CancelBurnEvent">CancelBurnEvent</a> {
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

<a name="0x0_Libra_remove_mint_capability"></a>

## Function `remove_mint_capability`

Removes and returns the
<code><a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> from
<code>account</code>.
Calls to this function will fail if
<code>account</code> does  not have a
published
<code><a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> resource at the top-level.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_mint_capability">remove_mint_capability</a>&lt;CoinType&gt;(account: &signer): <a href="#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_mint_capability">remove_mint_capability</a>&lt;CoinType&gt;(account: &signer): <a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;
<b>acquires</b> <a href="#0x0_Libra_MintCapability">MintCapability</a> {
    move_from&lt;<a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account))
}
</code></pre>



</details>

<a name="0x0_Libra_remove_burn_capability"></a>

## Function `remove_burn_capability`

Removes and returns the
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> from
<code>account</code>.
Calls to this function will fail if
<code>account</code> does  not have a
published
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> resource at the top-level.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_burn_capability">remove_burn_capability</a>&lt;CoinType&gt;(account: &signer): <a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_burn_capability">remove_burn_capability</a>&lt;CoinType&gt;(account: &signer): <a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;
<b>acquires</b> <a href="#0x0_Libra_BurnCapability">BurnCapability</a> {
    move_from&lt;<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account))
}
</code></pre>



</details>

<a name="0x0_Libra_preburn_value"></a>

## Function `preburn_value`

Returns the total value of
<code><a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;</code> that is waiting to be
burned throughout the system (i.e. the sum of all outstanding
preburn requests across all preburn resources for the
<code>CoinType</code>
currency).


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_value">preburn_value</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_value">preburn_value</a>&lt;CoinType&gt;(): u64 <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).preburn_value
}
</code></pre>



</details>

<a name="0x0_Libra_zero"></a>

## Function `zero`

Create a new
<code><a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;</code> with a value of
<code>0</code>. Anyone can call
this and it will be successful as long as
<code>CoinType</code> is a registered currency.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_zero">zero</a>&lt;CoinType&gt;(): <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_zero">zero</a>&lt;CoinType&gt;(): <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt; {
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;();
    <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt; { value: 0 }
}
</code></pre>



</details>

<a name="0x0_Libra_value"></a>

## Function `value`

Returns the
<code>value</code> of the passed in
<code>coin</code>. The value is
represented in the base units for the currency represented by
<code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_value">value</a>&lt;CoinType&gt;(coin: &<a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_value">value</a>&lt;CoinType&gt;(coin: &<a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;): u64 {
    coin.value
}
</code></pre>



</details>

<a name="0x0_Libra_split"></a>

## Function `split`

Removes
<code>amount</code> of value from the passed in
<code>coin</code>. Returns the
remaining balance of the passed in
<code>coin</code>, along with another coin
with value equal to
<code>amount</code>. Calls will fail if
<code>amount &gt; <a href="#0x0_Libra_value">Libra::value</a>(&coin)</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_split">split</a>&lt;CoinType&gt;(coin: <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, amount: u64): (<a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_split">split</a>&lt;CoinType&gt;(coin: <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;, amount: u64): (<a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;, <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;) {
    <b>let</b> other = <a href="#0x0_Libra_withdraw">withdraw</a>(&<b>mut</b> coin, amount);
    (coin, other)
}
</code></pre>



</details>

<a name="0x0_Libra_withdraw"></a>

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


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_withdraw">withdraw</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, amount: u64): <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_withdraw">withdraw</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;, amount: u64): <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt; {
    // Check that `amount` is less than the coin's value
    Transaction::assert(coin.value &gt;= amount, 10);
    coin.value = coin.value - amount;
    <a href="#0x0_Libra">Libra</a> { value: amount }
}
</code></pre>



</details>

<a name="0x0_Libra_join"></a>

## Function `join`

Combines the two coins of the same currency
<code>CoinType</code> passed-in,
and returns a new coin whose value is equal to the sum of the two inputs.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_join">join</a>&lt;CoinType&gt;(coin1: <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, coin2: <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;): <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_join">join</a>&lt;CoinType&gt;(coin1: <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;, coin2: <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;): <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;  {
    <a href="#0x0_Libra_deposit">deposit</a>(&<b>mut</b> coin1, coin2);
    coin1
}
</code></pre>



</details>

<a name="0x0_Libra_deposit"></a>

## Function `deposit`

"Merges" the two coins.
The coin passed in by reference will have a value equal to the sum of the two coins
The
<code>check</code> coin is consumed in the process


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_deposit">deposit</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;, check: <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_deposit">deposit</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;, check: <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="#0x0_Libra">Libra</a> { value } = check;
    coin.value = coin.value + value;
}
</code></pre>



</details>

<a name="0x0_Libra_destroy_zero"></a>

## Function `destroy_zero`

Destroy a zero-value coin. Calls will fail if the
<code>value</code> in the passed-in
<code>coin</code> is non-zero
The amount of
<code><a href="#0x0_Libra">Libra</a></code> in the system is a tightly controlled property,
so you cannot "burn" any non-zero amount of
<code><a href="#0x0_Libra">Libra</a></code> without having
a
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a></code> for the specific
<code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(coin: <a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(coin: <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="#0x0_Libra">Libra</a> { value } = coin;
    Transaction::assert(value == 0, 5)
}
</code></pre>



</details>

<a name="0x0_Libra_register_currency"></a>

## Function `register_currency`

Register the type
<code>CoinType</code> as a currency. Until the type is
registered as a currency it cannot be used as a coin/currency unit in Libra.
The passed-in
<code>account</code> must be a specific address (
<code><a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code>) and
the
<code>account</code> must also have the correct
<code><a href="#0x0_Libra_AddCurrency">AddCurrency</a></code> association
privilege. After the first registration of
<code>CoinType</code> as a
currency, all subsequent tries to register
<code>CoinType</code> as a currency
will fail.
When the
<code>CoinType</code> is registered it publishes the
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;</code> resource under the
<code><a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()</code> and
adds the currency to the set of
<code><a href="RegisteredCurrencies.md#0x0_RegisteredCurrencies">RegisteredCurrencies</a></code>. It returns
<code><a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;</code> and
<code><a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;</code> resources.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(account: &signer, to_lbr_exchange_rate: <a href="FixedPoint32.md#0x0_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>, is_synthetic: bool, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;): (<a href="#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;, <a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(
    account: &signer,
    to_lbr_exchange_rate: <a href="FixedPoint32.md#0x0_FixedPoint32">FixedPoint32</a>,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: vector&lt;u8&gt;,
): (<a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;, <a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;)
<b>acquires</b> <a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a> {
    // And only callable by the designated currency address.
    Transaction::assert(
        <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>() &&
        <a href="Association.md#0x0_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x0_Libra_AddCurrency">AddCurrency</a>&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account)),
        8
    );

    move_to(account, <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt; {
        total_value: 0,
        preburn_value: 0,
        to_lbr_exchange_rate,
        is_synthetic,
        scaling_factor,
        fractional_part,
        currency_code: <b>copy</b> currency_code,
        can_mint: <b>true</b>,
        mint_events: <a href="Event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_Libra_MintEvent">MintEvent</a>&gt;(account),
        burn_events: <a href="Event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_Libra_BurnEvent">BurnEvent</a>&gt;(account),
        preburn_events: <a href="Event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_Libra_PreburnEvent">PreburnEvent</a>&gt;(account),
        cancel_burn_events: <a href="Event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_Libra_CancelBurnEvent">CancelBurnEvent</a>&gt;(account)
    });
    <a href="RegisteredCurrencies.md#0x0_RegisteredCurrencies_add_currency_code">RegisteredCurrencies::add_currency_code</a>(
        currency_code,
        &borrow_global&lt;<a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a>&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>()).cap
    );
    (<a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;{}, <a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;{})
}
</code></pre>



</details>

<a name="0x0_Libra_market_cap"></a>

## Function `market_cap`

Returns the total amount of currency minted of type
<code>CoinType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_market_cap">market_cap</a>&lt;CoinType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_market_cap">market_cap</a>&lt;CoinType&gt;(): u128
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).total_value
}
</code></pre>



</details>

<a name="0x0_Libra_approx_lbr_for_value"></a>

## Function `approx_lbr_for_value`

Returns the value of the coin in the
<code>FromCoinType</code> currency in LBR.
This should only be used where a _rough_ approximation of the exchange
rate is needed.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value: u64): u64
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> lbr_exchange_rate = <a href="#0x0_Libra_lbr_exchange_rate">lbr_exchange_rate</a>&lt;FromCoinType&gt;();
    <a href="FixedPoint32.md#0x0_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(from_value, lbr_exchange_rate)
}
</code></pre>



</details>

<a name="0x0_Libra_approx_lbr_for_coin"></a>

## Function `approx_lbr_for_coin`

Returns the value of the coin in the
<code>FromCoinType</code> currency in LBR.
This should only be used where a rough approximation of the exchange
rate is needed.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_approx_lbr_for_coin">approx_lbr_for_coin</a>&lt;FromCoinType&gt;(coin: &<a href="#0x0_Libra_Libra">Libra::Libra</a>&lt;FromCoinType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_approx_lbr_for_coin">approx_lbr_for_coin</a>&lt;FromCoinType&gt;(coin: &<a href="#0x0_Libra">Libra</a>&lt;FromCoinType&gt;): u64
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> from_value = <a href="#0x0_Libra_value">value</a>(coin);
    <a href="#0x0_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value)
}
</code></pre>



</details>

<a name="0x0_Libra_is_currency"></a>

## Function `is_currency`

Returns
<code><b>true</b></code> if the type
<code>CoinType</code> is a registered currency.
Returns
<code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(): bool {
    exists&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x0_Libra_is_synthetic_currency"></a>

## Function `is_synthetic_currency`

Returns
<code><b>true</b></code> if
<code>CoinType</code> is a synthetic currency as defined in
its
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code>. Returns
<code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(): bool
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>();
    exists&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(addr) &&
        borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(addr).is_synthetic
}
</code></pre>



</details>

<a name="0x0_Libra_scaling_factor"></a>

## Function `scaling_factor`

Returns the scaling factor for the
<code>CoinType</code> currency as defined
in its
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_scaling_factor">scaling_factor</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_scaling_factor">scaling_factor</a>&lt;CoinType&gt;(): u64
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).scaling_factor
}
</code></pre>



</details>

<a name="0x0_Libra_fractional_part"></a>

## Function `fractional_part`

Returns the representable (i.e. real-world) fractional part for the
<code>CoinType</code> currency as defined in its
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_fractional_part">fractional_part</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_fractional_part">fractional_part</a>&lt;CoinType&gt;(): u64
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).fractional_part
}
</code></pre>



</details>

<a name="0x0_Libra_currency_code"></a>

## Function `currency_code`

Returns the currency code for the registered currency as defined in
its
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_currency_code">currency_code</a>&lt;CoinType&gt;(): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_currency_code">currency_code</a>&lt;CoinType&gt;(): vector&lt;u8&gt;
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    *&borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).currency_code
}
</code></pre>



</details>

<a name="0x0_Libra_update_lbr_exchange_rate"></a>

## Function `update_lbr_exchange_rate`

Updates the
<code>to_lbr_exchange_rate</code> held in the
<code><a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a></code> for
<code>FromCoinType</code> to the new passed-in
<code>lbr_exchange_rate</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_update_lbr_exchange_rate">update_lbr_exchange_rate</a>&lt;FromCoinType&gt;(account: &signer, lbr_exchange_rate: <a href="FixedPoint32.md#0x0_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_update_lbr_exchange_rate">update_lbr_exchange_rate</a>&lt;FromCoinType&gt;(
    account: &signer,
    lbr_exchange_rate: <a href="FixedPoint32.md#0x0_FixedPoint32">FixedPoint32</a>
) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="Association.md#0x0_Association_assert_account_is_blessed">Association::assert_account_is_blessed</a>(account);
    <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;FromCoinType&gt;(account);
    <b>let</b> currency_info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;FromCoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    currency_info.to_lbr_exchange_rate = lbr_exchange_rate;
}
</code></pre>



</details>

<a name="0x0_Libra_lbr_exchange_rate"></a>

## Function `lbr_exchange_rate`

Returns the (rough) exchange rate between
<code>CoinType</code> and
<code><a href="LBR.md#0x0_LBR">LBR</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_lbr_exchange_rate">lbr_exchange_rate</a>&lt;CoinType&gt;(): <a href="FixedPoint32.md#0x0_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_lbr_exchange_rate">lbr_exchange_rate</a>&lt;CoinType&gt;(): <a href="FixedPoint32.md#0x0_FixedPoint32">FixedPoint32</a>
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    *&borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).to_lbr_exchange_rate
}
</code></pre>



</details>

<a name="0x0_Libra_update_minting_ability"></a>

## Function `update_minting_ability`

There may be situations in which we disallow the further minting of
coins in the system without removing the currency. This function
allows the association to control whether or not further coins of
<code>CoinType</code> can be minted or not. If this is called with
<code>can_mint =
<b>true</b></code>, then minting is allowed, if
<code>can_mint = <b>false</b></code> then minting is
disallowed until it is turned back on via this function. All coins
start out in the default state of
<code>can_mint = <b>true</b></code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_update_minting_ability">update_minting_ability</a>&lt;CoinType&gt;(account: &signer, can_mint: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_update_minting_ability">update_minting_ability</a>&lt;CoinType&gt;(account: &signer, can_mint: bool)
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;(account);
    <b>let</b> currency_info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
    currency_info.can_mint = can_mint;
}
</code></pre>



</details>

<a name="0x0_Libra_assert_assoc_and_currency"></a>

## Function `assert_assoc_and_currency`

Asserts that the
<code>account</code> is an association account, and that
<code>CoinType</code> is a registered currency type.


<pre><code><b>fun</b> <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;(account: &signer) {
    <a href="Association.md#0x0_Association_assert_is_association">Association::assert_is_association</a>(account);
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;();
}
</code></pre>



</details>

<a name="0x0_Libra_assert_is_coin"></a>

## Function `assert_is_coin`

Asserts that
<code>CoinType</code> is a registered currency.


<pre><code><b>fun</b> <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;() {
    Transaction::assert(<a href="#0x0_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(), 1);
}
</code></pre>



</details>

<a name="0x0_Libra_Specification"></a>

## Specification



<pre><code>pragma verify = <b>false</b>;
</code></pre>




<a name="0x0_Libra_spec_currency_addr"></a>


<pre><code><b>define</b> <a href="#0x0_Libra_spec_currency_addr">spec_currency_addr</a>(): address { 0xA550C18 }
<a name="0x0_Libra_spec_is_currency"></a>
<b>define</b> <a href="#0x0_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;(): bool {
    exists&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="#0x0_Libra_spec_currency_addr">spec_currency_addr</a>())
}
</code></pre>



<a name="0x0_Libra_Specification_Libra"></a>

### Struct `Libra`


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra">Libra</a>&lt;CoinType&gt;
</code></pre>



<dl>
<dt>

<code>value: u64</code>
</dt>
<dd>
 The value of this coin in the base units for
<code>CoinType</code>
</dd>
</dl>



<pre><code><b>invariant</b> <b>pack</b> <a href="#0x0_Libra_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; = <a href="#0x0_Libra_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; + value;
<b>invariant</b> <b>unpack</b> <a href="#0x0_Libra_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; = <a href="#0x0_Libra_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; - value;
</code></pre>



<a name="0x0_Libra_Specification_register_currency"></a>

### Function `register_currency`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(account: &signer, to_lbr_exchange_rate: <a href="FixedPoint32.md#0x0_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>, is_synthetic: bool, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;): (<a href="#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;, <a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>ensures</b> <a href="#0x0_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;();
</code></pre>



<a name="0x0_Libra_Specification_is_currency"></a>

### Function `is_currency`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(): bool
</code></pre>




<pre><code><b>ensures</b> result == <a href="#0x0_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;();
</code></pre>



<a name="0x0_Libra_Specification_assert_is_coin"></a>

### Function `assert_is_coin`


<pre><code><b>fun</b> <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;()
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x0_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;();
</code></pre>




<a name="0x0_Libra_sum_of_coin_values"></a>


<pre><code><b>global</b> <a href="#0x0_Libra_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt;: num;
</code></pre>
