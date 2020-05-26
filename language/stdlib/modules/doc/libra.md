
<a name="0x0_Libra"></a>

# Module `0x0::Libra`

### Table of Contents

-  [Struct `T`](#0x0_Libra_T)
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
-  [Function `grant_mint_capability`](#0x0_Libra_grant_mint_capability)
-  [Function `grant_mint_capability_for_sender`](#0x0_Libra_grant_mint_capability_for_sender)
-  [Function `grant_burn_capability`](#0x0_Libra_grant_burn_capability)
-  [Function `grant_burn_capability_for_sender`](#0x0_Libra_grant_burn_capability_for_sender)
-  [Function `mint`](#0x0_Libra_mint)
-  [Function `burn`](#0x0_Libra_burn)
-  [Function `cancel_burn`](#0x0_Libra_cancel_burn)
-  [Function `new_preburn`](#0x0_Libra_new_preburn)
-  [Function `mint_with_capability`](#0x0_Libra_mint_with_capability)
-  [Function `new_preburn_with_capability`](#0x0_Libra_new_preburn_with_capability)
-  [Function `preburn_with_resource`](#0x0_Libra_preburn_with_resource)
-  [Function `preburn_to_sender`](#0x0_Libra_preburn_to_sender)
-  [Function `burn_with_capability`](#0x0_Libra_burn_with_capability)
-  [Function `burn_with_resource_cap`](#0x0_Libra_burn_with_resource_cap)
-  [Function `cancel_burn_with_capability`](#0x0_Libra_cancel_burn_with_capability)
-  [Function `publish_preburn`](#0x0_Libra_publish_preburn)
-  [Function `publish_mint_capability`](#0x0_Libra_publish_mint_capability)
-  [Function `remove_preburn`](#0x0_Libra_remove_preburn)
-  [Function `destroy_preburn`](#0x0_Libra_destroy_preburn)
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
-  [Function `currency_addr`](#0x0_Libra_currency_addr)
-  [Function `assert_assoc_and_currency`](#0x0_Libra_assert_assoc_and_currency)
-  [Function `assert_is_coin`](#0x0_Libra_assert_is_coin)
-  [Specification](#0x0_Libra_Specification)
    -  [Struct `T`](#0x0_Libra_Specification_T)
    -  [Function `register_currency`](#0x0_Libra_Specification_register_currency)
    -  [Function `is_currency`](#0x0_Libra_Specification_is_currency)
    -  [Function `assert_is_coin`](#0x0_Libra_Specification_assert_is_coin)



<a name="0x0_Libra_T"></a>

## Struct `T`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>value: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Libra_MintCapability"></a>

## Struct `MintCapability`



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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="registered_currencies.md#0x0_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Libra_MintEvent"></a>

## Struct `MintEvent`



<pre><code><b>struct</b> <a href="#0x0_Libra_MintEvent">MintEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Libra_BurnEvent"></a>

## Struct `BurnEvent`



<pre><code><b>struct</b> <a href="#0x0_Libra_BurnEvent">BurnEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>preburn_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Libra_PreburnEvent"></a>

## Struct `PreburnEvent`



<pre><code><b>struct</b> <a href="#0x0_Libra_PreburnEvent">PreburnEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>preburn_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Libra_CancelBurnEvent"></a>

## Struct `CancelBurnEvent`



<pre><code><b>struct</b> <a href="#0x0_Libra_CancelBurnEvent">CancelBurnEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>preburn_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Libra_CurrencyInfo"></a>

## Struct `CurrencyInfo`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>total_value: u128</code>
</dt>
<dd>

</dd>
<dt>

<code>preburn_value: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>to_lbr_exchange_rate: <a href="fixedpoint32.md#0x0_FixedPoint32_T">FixedPoint32::T</a></code>
</dt>
<dd>

</dd>
<dt>

<code>is_synthetic: bool</code>
</dt>
<dd>

</dd>
<dt>

<code>scaling_factor: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>fractional_part: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>can_mint: bool</code>
</dt>
<dd>

</dd>
<dt>

<code>mint_events: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_Libra_MintEvent">Libra::MintEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>burn_events: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_Libra_BurnEvent">Libra::BurnEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>preburn_events: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_Libra_PreburnEvent">Libra::PreburnEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>cancel_burn_events: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_Libra_CancelBurnEvent">Libra::CancelBurnEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Libra_Preburn"></a>

## Struct `Preburn`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>requests: vector&lt;<a href="#0x0_Libra_T">Libra::T</a>&lt;Token&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>is_approved: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Libra_AddCurrency"></a>

## Struct `AddCurrency`



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



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_initialize">initialize</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_initialize">initialize</a>() {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    Transaction::assert(Transaction::sender() == <a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>(), 0);
    <b>let</b> cap = <a href="registered_currencies.md#0x0_RegisteredCurrencies_initialize">RegisteredCurrencies::initialize</a>();
    move_to_sender(<a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a>{ cap })
}
</code></pre>



</details>

<a name="0x0_Libra_grant_mint_capability"></a>

## Function `grant_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_mint_capability">grant_mint_capability</a>&lt;CoinType&gt;(): <a href="#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_mint_capability">grant_mint_capability</a>&lt;CoinType&gt;(): <a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt; {
    <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;();
    <a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt; { }
}
</code></pre>



</details>

<a name="0x0_Libra_grant_mint_capability_for_sender"></a>

## Function `grant_mint_capability_for_sender`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_mint_capability_for_sender">grant_mint_capability_for_sender</a>&lt;CoinType&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_mint_capability_for_sender">grant_mint_capability_for_sender</a>&lt;CoinType&gt;() {
    Transaction::assert(Transaction::sender() == 0xB1E55ED, 0);
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;();
    move_to_sender(<a href="#0x0_Libra_grant_mint_capability">grant_mint_capability</a>&lt;CoinType&gt;());
}
</code></pre>



</details>

<a name="0x0_Libra_grant_burn_capability"></a>

## Function `grant_burn_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_burn_capability">grant_burn_capability</a>&lt;CoinType&gt;(): <a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_burn_capability">grant_burn_capability</a>&lt;CoinType&gt;(): <a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt; {
    <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;();
    <a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt; { }
}
</code></pre>



</details>

<a name="0x0_Libra_grant_burn_capability_for_sender"></a>

## Function `grant_burn_capability_for_sender`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_burn_capability_for_sender">grant_burn_capability_for_sender</a>&lt;CoinType&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_grant_burn_capability_for_sender">grant_burn_capability_for_sender</a>&lt;CoinType&gt;() {
    Transaction::assert(Transaction::sender() == 0xB1E55ED, 0);
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;();
    move_to_sender(<a href="#0x0_Libra_grant_burn_capability">grant_burn_capability</a>&lt;CoinType&gt;());
}
</code></pre>



</details>

<a name="0x0_Libra_mint"></a>

## Function `mint`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_mint">mint</a>&lt;Token&gt;(amount: u64): <a href="#0x0_Libra_T">Libra::T</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_mint">mint</a>&lt;Token&gt;(amount: u64): <a href="#0x0_Libra_T">T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_MintCapability">MintCapability</a> {
    <a href="#0x0_Libra_mint_with_capability">mint_with_capability</a>(amount, borrow_global&lt;<a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;Token&gt;&gt;(Transaction::sender()))
}
</code></pre>



</details>

<a name="0x0_Libra_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn">burn</a>&lt;Token&gt;(preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn">burn</a>&lt;Token&gt;(
    preburn_address: address
) <b>acquires</b> <a href="#0x0_Libra_BurnCapability">BurnCapability</a>, <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    <a href="#0x0_Libra_burn_with_capability">burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;Token&gt;&gt;(Transaction::sender())
    )
}
</code></pre>



</details>

<a name="0x0_Libra_cancel_burn"></a>

## Function `cancel_burn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_cancel_burn">cancel_burn</a>&lt;Token&gt;(preburn_address: address): <a href="#0x0_Libra_T">Libra::T</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_cancel_burn">cancel_burn</a>&lt;Token&gt;(
    preburn_address: address
): <a href="#0x0_Libra_T">T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_Libra_BurnCapability">BurnCapability</a>, <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    <a href="#0x0_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;Token&gt;&gt;(Transaction::sender())
    )
}
</code></pre>



</details>

<a name="0x0_Libra_new_preburn"></a>

## Function `new_preburn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_new_preburn">new_preburn</a>&lt;Token&gt;(): <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_new_preburn">new_preburn</a>&lt;Token&gt;(): <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt; {
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;Token&gt;();
    <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt; { requests: <a href="vector.md#0x0_Vector_empty">Vector::empty</a>(), is_approved: <b>false</b>, }
}
</code></pre>



</details>

<a name="0x0_Libra_mint_with_capability"></a>

## Function `mint_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_mint_with_capability">mint_with_capability</a>&lt;Token&gt;(value: u64, _capability: &<a href="#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;Token&gt;): <a href="#0x0_Libra_T">Libra::T</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_mint_with_capability">mint_with_capability</a>&lt;Token&gt;(
    value: u64,
    _capability: &<a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;Token&gt;
): <a href="#0x0_Libra_T">T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;Token&gt;();
    // TODO: temporary measure for testnet only: limit minting <b>to</b> 1B <a href="#0x0_Libra">Libra</a> at a time.
    // this is <b>to</b> prevent the market cap's total value from hitting u64_max due <b>to</b> excessive
    // minting. This will not be a problem in the production <a href="#0x0_Libra">Libra</a> system because coins will
    // be backed with real-world assets, and thus minting will be correspondingly rarer.
    // * 1000000 here because the unit is microlibra
    Transaction::assert(<a href="#0x0_Libra_value">value</a> &lt;= 1000000000 * 1000000, 11);
    <b>let</b> currency_code = <a href="#0x0_Libra_currency_code">currency_code</a>&lt;Token&gt;();
    // <b>update</b> market cap <b>resource</b> <b>to</b> reflect minting
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;Token&gt;&gt;(0xA550C18);
    Transaction::assert(info.can_mint, 4);
    info.total_value = info.total_value + (value <b>as</b> u128);
    // don't emit mint events for synthetic currenices
    <b>if</b> (!info.is_synthetic) {
        <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> info.mint_events,
            <a href="#0x0_Libra_MintEvent">MintEvent</a>{
                amount: value,
                currency_code,
            }
        );
    };

    <a href="#0x0_Libra_T">T</a>&lt;Token&gt; { value }
}
</code></pre>



</details>

<a name="0x0_Libra_new_preburn_with_capability"></a>

## Function `new_preburn_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_new_preburn_with_capability">new_preburn_with_capability</a>&lt;Token&gt;(_capability: &<a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;): <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_new_preburn_with_capability">new_preburn_with_capability</a>&lt;Token&gt;(
    _capability: &<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;Token&gt;
): <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt; {
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;Token&gt;();
    <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt; { requests: <a href="vector.md#0x0_Vector_empty">Vector::empty</a>(), is_approved: <b>true</b> }
}
</code></pre>



</details>

<a name="0x0_Libra_preburn_with_resource"></a>

## Function `preburn_with_resource`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_with_resource">preburn_with_resource</a>&lt;Token&gt;(coin: <a href="#0x0_Libra_T">Libra::T</a>&lt;Token&gt;, preburn: &<b>mut</b> <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_with_resource">preburn_with_resource</a>&lt;Token&gt;(
    coin: <a href="#0x0_Libra_T">T</a>&lt;Token&gt;,
    preburn: &<b>mut</b> <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt;,
    preburn_address: address,
) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> coin_value = <a href="#0x0_Libra_value">value</a>(&coin);
    <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(
        &<b>mut</b> preburn.requests,
        coin
    );
    <b>let</b> currency_code = <a href="#0x0_Libra_currency_code">currency_code</a>&lt;Token&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;Token&gt;&gt;(0xA550C18);
    info.preburn_value = info.preburn_value + coin_value;
    // don't emit preburn events for synthetic currencies
    <b>if</b> (!info.is_synthetic) {
        <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>(
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

<a name="0x0_Libra_preburn_to_sender"></a>

## Function `preburn_to_sender`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_to_sender">preburn_to_sender</a>&lt;Token&gt;(coin: <a href="#0x0_Libra_T">Libra::T</a>&lt;Token&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_to_sender">preburn_to_sender</a>&lt;Token&gt;(coin: <a href="#0x0_Libra_T">T</a>&lt;Token&gt;) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    <b>let</b> sender = Transaction::sender();
    <a href="#0x0_Libra_preburn_with_resource">preburn_with_resource</a>(coin, borrow_global_mut&lt;<a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt;&gt;(sender), sender);
}
</code></pre>



</details>

<a name="0x0_Libra_burn_with_capability"></a>

## Function `burn_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn_with_capability">burn_with_capability</a>&lt;Token&gt;(preburn_address: address, capability: &<a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn_with_capability">burn_with_capability</a>&lt;Token&gt;(
    preburn_address: address,
    capability: &<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;Token&gt;
) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    // destroy the coin at the head of the preburn queue
    <a href="#0x0_Libra_burn_with_resource_cap">burn_with_resource_cap</a>(
        borrow_global_mut&lt;<a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address),
        preburn_address,
        capability
    )
}
</code></pre>



</details>

<a name="0x0_Libra_burn_with_resource_cap"></a>

## Function `burn_with_resource_cap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn_with_resource_cap">burn_with_resource_cap</a>&lt;Token&gt;(preburn: &<b>mut</b> <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;, preburn_address: address, _capability: &<a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_burn_with_resource_cap">burn_with_resource_cap</a>&lt;Token&gt;(
    preburn: &<b>mut</b> <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt;,
    preburn_address: address,
    _capability: &<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;Token&gt;
) <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    // destroy the coin at the head of the preburn queue
    <b>let</b> <a href="#0x0_Libra_T">T</a> { value } = <a href="vector.md#0x0_Vector_remove">Vector::remove</a>(&<b>mut</b> preburn.requests, 0);
    // <b>update</b> the market cap
    <b>let</b> currency_code = <a href="#0x0_Libra_currency_code">currency_code</a>&lt;Token&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;Token&gt;&gt;(0xA550C18);
    info.total_value = info.total_value - (value <b>as</b> u128);
    info.preburn_value = info.preburn_value - value;
    // don't emit burn events for synthetic currencies
    <b>if</b> (!info.is_synthetic) {
        <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>(
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



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;Token&gt;(preburn_address: address, _capability: &<a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;): <a href="#0x0_Libra_T">Libra::T</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;Token&gt;(
    preburn_address: address,
    _capability: &<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;Token&gt;
): <a href="#0x0_Libra_T">T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>, <a href="#0x0_Libra_Preburn">Preburn</a> {
    // destroy the coin at the head of the preburn queue
    <b>let</b> preburn = borrow_global_mut&lt;<a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address);
    <b>let</b> coin = <a href="vector.md#0x0_Vector_remove">Vector::remove</a>(&<b>mut</b> preburn.requests, 0);
    // <b>update</b> the market cap
    <b>let</b> currency_code = <a href="#0x0_Libra_currency_code">currency_code</a>&lt;Token&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;Token&gt;&gt;(0xA550C18);
    <b>let</b> amount = <a href="#0x0_Libra_value">value</a>(&coin);
    info.preburn_value = info.preburn_value - amount;
    // Don't emit cancel burn events for synthetic currencies. cancel burn shouldn't be be used
    // for synthetics in the first place
    <b>if</b> (!info.is_synthetic) {
        <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>(
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

<a name="0x0_Libra_publish_preburn"></a>

## Function `publish_preburn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_preburn">publish_preburn</a>&lt;Token&gt;(preburn: <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_preburn">publish_preburn</a>&lt;Token&gt;(preburn: <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt;) {
    move_to_sender(preburn)
}
</code></pre>



</details>

<a name="0x0_Libra_publish_mint_capability"></a>

## Function `publish_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_mint_capability">publish_mint_capability</a>&lt;Token&gt;(capability: <a href="#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;Token&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_publish_mint_capability">publish_mint_capability</a>&lt;Token&gt;(capability: <a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;Token&gt;) {
    move_to_sender(capability)
}
</code></pre>



</details>

<a name="0x0_Libra_remove_preburn"></a>

## Function `remove_preburn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_preburn">remove_preburn</a>&lt;Token&gt;(): <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_preburn">remove_preburn</a>&lt;Token&gt;(): <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_Libra_Preburn">Preburn</a> {
    move_from&lt;<a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt;&gt;(Transaction::sender())
}
</code></pre>



</details>

<a name="0x0_Libra_destroy_preburn"></a>

## Function `destroy_preburn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_destroy_preburn">destroy_preburn</a>&lt;Token&gt;(preburn: <a href="#0x0_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_destroy_preburn">destroy_preburn</a>&lt;Token&gt;(preburn: <a href="#0x0_Libra_Preburn">Preburn</a>&lt;Token&gt;) {
    <b>let</b> <a href="#0x0_Libra_Preburn">Preburn</a> { requests, is_approved: _ } = preburn;
    <a href="vector.md#0x0_Vector_destroy_empty">Vector::destroy_empty</a>(requests)
}
</code></pre>



</details>

<a name="0x0_Libra_remove_mint_capability"></a>

## Function `remove_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_mint_capability">remove_mint_capability</a>&lt;Token&gt;(): <a href="#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_mint_capability">remove_mint_capability</a>&lt;Token&gt;(): <a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_Libra_MintCapability">MintCapability</a> {
    move_from&lt;<a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;Token&gt;&gt;(Transaction::sender())
}
</code></pre>



</details>

<a name="0x0_Libra_remove_burn_capability"></a>

## Function `remove_burn_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_burn_capability">remove_burn_capability</a>&lt;Token&gt;(): <a href="#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_remove_burn_capability">remove_burn_capability</a>&lt;Token&gt;(): <a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_Libra_BurnCapability">BurnCapability</a> {
    move_from&lt;<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;Token&gt;&gt;(Transaction::sender())
}
</code></pre>



</details>

<a name="0x0_Libra_preburn_value"></a>

## Function `preburn_value`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_value">preburn_value</a>&lt;Token&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_preburn_value">preburn_value</a>&lt;Token&gt;(): u64 <b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;Token&gt;&gt;(0xA550C18).preburn_value
}
</code></pre>



</details>

<a name="0x0_Libra_zero"></a>

## Function `zero`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_zero">zero</a>&lt;CoinType&gt;(): <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_zero">zero</a>&lt;CoinType&gt;(): <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt; {
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;();
    <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt; { value: 0 }
}
</code></pre>



</details>

<a name="0x0_Libra_value"></a>

## Function `value`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_value">value</a>&lt;CoinType&gt;(coin: &<a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_value">value</a>&lt;CoinType&gt;(coin: &<a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;): u64 {
    coin.value
}
</code></pre>



</details>

<a name="0x0_Libra_split"></a>

## Function `split`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_split">split</a>&lt;CoinType&gt;(coin: <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;, amount: u64): (<a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;, <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_split">split</a>&lt;CoinType&gt;(coin: <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;, amount: u64): (<a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;, <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;) {
    <b>let</b> other = <a href="#0x0_Libra_withdraw">withdraw</a>(&<b>mut</b> coin, amount);
    (coin, other)
}
</code></pre>



</details>

<a name="0x0_Libra_withdraw"></a>

## Function `withdraw`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_withdraw">withdraw</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;, amount: u64): <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_withdraw">withdraw</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;, amount: u64): <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt; {
    // Check that `amount` is less than the coin's value
    Transaction::assert(coin.value &gt;= amount, 10);
    coin.value = coin.value - amount;
    <a href="#0x0_Libra_T">T</a> { value: amount }
}
</code></pre>



</details>

<a name="0x0_Libra_join"></a>

## Function `join`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_join">join</a>&lt;CoinType&gt;(coin1: <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;, coin2: <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;): <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_join">join</a>&lt;CoinType&gt;(coin1: <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;, coin2: <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;): <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;  {
    <a href="#0x0_Libra_deposit">deposit</a>(&<b>mut</b> coin1, coin2);
    coin1
}
</code></pre>



</details>

<a name="0x0_Libra_deposit"></a>

## Function `deposit`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_deposit">deposit</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;, check: <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_deposit">deposit</a>&lt;CoinType&gt;(coin: &<b>mut</b> <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;, check: <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="#0x0_Libra_T">T</a> { value } = check;
    coin.value = coin.value + value;
}
</code></pre>



</details>

<a name="0x0_Libra_destroy_zero"></a>

## Function `destroy_zero`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(coin: <a href="#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_destroy_zero">destroy_zero</a>&lt;CoinType&gt;(coin: <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;) {
    <b>let</b> <a href="#0x0_Libra_T">T</a> { value } = coin;
    Transaction::assert(value == 0, 5)
}
</code></pre>



</details>

<a name="0x0_Libra_register_currency"></a>

## Function `register_currency`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(to_lbr_exchange_rate: <a href="fixedpoint32.md#0x0_FixedPoint32_T">FixedPoint32::T</a>, is_synthetic: bool, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(
    to_lbr_exchange_rate: <a href="fixedpoint32.md#0x0_FixedPoint32_T">FixedPoint32::T</a>,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a> {
    // And only callable by the designated currency address.
    Transaction::assert(<a href="association.md#0x0_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x0_Libra_AddCurrency">AddCurrency</a>&gt;(Transaction::sender()), 8);

    move_to_sender(<a href="#0x0_Libra_MintCapability">MintCapability</a>&lt;CoinType&gt;{});
    move_to_sender(<a href="#0x0_Libra_BurnCapability">BurnCapability</a>&lt;CoinType&gt;{});
    move_to_sender(<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt; {
        total_value: 0,
        preburn_value: 0,
        to_lbr_exchange_rate,
        is_synthetic,
        scaling_factor,
        fractional_part,
        currency_code: <b>copy</b> currency_code,
        can_mint: <b>true</b>,
        mint_events: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_Libra_MintEvent">MintEvent</a>&gt;(),
        burn_events: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_Libra_BurnEvent">BurnEvent</a>&gt;(),
        preburn_events: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_Libra_PreburnEvent">PreburnEvent</a>&gt;(),
        cancel_burn_events: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_Libra_CancelBurnEvent">CancelBurnEvent</a>&gt;()
    });
    <a href="registered_currencies.md#0x0_RegisteredCurrencies_add_currency_code">RegisteredCurrencies::add_currency_code</a>(
        currency_code,
        &borrow_global&lt;<a href="#0x0_Libra_CurrencyRegistrationCapability">CurrencyRegistrationCapability</a>&gt;(<a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>()).cap
    )
}
</code></pre>



</details>

<a name="0x0_Libra_market_cap"></a>

## Function `market_cap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_market_cap">market_cap</a>&lt;CoinType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_market_cap">market_cap</a>&lt;CoinType&gt;(): u128
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="#0x0_Libra_currency_addr">currency_addr</a>()).total_value
}
</code></pre>



</details>

<a name="0x0_Libra_approx_lbr_for_value"></a>

## Function `approx_lbr_for_value`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value: u64): u64
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> lbr_exchange_rate = <a href="#0x0_Libra_lbr_exchange_rate">lbr_exchange_rate</a>&lt;FromCoinType&gt;();
    <a href="fixedpoint32.md#0x0_FixedPoint32_multiply_u64">FixedPoint32::multiply_u64</a>(from_value, lbr_exchange_rate)
}
</code></pre>



</details>

<a name="0x0_Libra_approx_lbr_for_coin"></a>

## Function `approx_lbr_for_coin`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_approx_lbr_for_coin">approx_lbr_for_coin</a>&lt;FromCoinType&gt;(coin: &<a href="#0x0_Libra_T">Libra::T</a>&lt;FromCoinType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_approx_lbr_for_coin">approx_lbr_for_coin</a>&lt;FromCoinType&gt;(coin: &<a href="#0x0_Libra_T">T</a>&lt;FromCoinType&gt;): u64
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> from_value = <a href="#0x0_Libra_value">value</a>(coin);
    <a href="#0x0_Libra_approx_lbr_for_value">approx_lbr_for_value</a>&lt;FromCoinType&gt;(from_value)
}
</code></pre>



</details>

<a name="0x0_Libra_is_currency"></a>

## Function `is_currency`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_is_currency">is_currency</a>&lt;CoinType&gt;(): bool {
    exists&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="#0x0_Libra_currency_addr">currency_addr</a>())
}
</code></pre>



</details>

<a name="0x0_Libra_is_synthetic_currency"></a>

## Function `is_synthetic_currency`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_is_synthetic_currency">is_synthetic_currency</a>&lt;CoinType&gt;(): bool
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <b>let</b> addr = <a href="#0x0_Libra_currency_addr">currency_addr</a>();
    exists&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(addr) &&
        borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(addr).is_synthetic
}
</code></pre>



</details>

<a name="0x0_Libra_scaling_factor"></a>

## Function `scaling_factor`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_scaling_factor">scaling_factor</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_scaling_factor">scaling_factor</a>&lt;CoinType&gt;(): u64
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="#0x0_Libra_currency_addr">currency_addr</a>()).scaling_factor
}
</code></pre>



</details>

<a name="0x0_Libra_fractional_part"></a>

## Function `fractional_part`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_fractional_part">fractional_part</a>&lt;CoinType&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_fractional_part">fractional_part</a>&lt;CoinType&gt;(): u64
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="#0x0_Libra_currency_addr">currency_addr</a>()).fractional_part
}
</code></pre>



</details>

<a name="0x0_Libra_currency_code"></a>

## Function `currency_code`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_currency_code">currency_code</a>&lt;CoinType&gt;(): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_currency_code">currency_code</a>&lt;CoinType&gt;(): vector&lt;u8&gt;
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    *&borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="#0x0_Libra_currency_addr">currency_addr</a>()).currency_code
}
</code></pre>



</details>

<a name="0x0_Libra_update_lbr_exchange_rate"></a>

## Function `update_lbr_exchange_rate`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_update_lbr_exchange_rate">update_lbr_exchange_rate</a>&lt;FromCoinType&gt;(lbr_exchange_rate: <a href="fixedpoint32.md#0x0_FixedPoint32_T">FixedPoint32::T</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_update_lbr_exchange_rate">update_lbr_exchange_rate</a>&lt;FromCoinType&gt;(lbr_exchange_rate: <a href="fixedpoint32.md#0x0_FixedPoint32_T">FixedPoint32::T</a>)
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;FromCoinType&gt;();
    <b>let</b> currency_info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;FromCoinType&gt;&gt;(<a href="#0x0_Libra_currency_addr">currency_addr</a>());
    currency_info.to_lbr_exchange_rate = lbr_exchange_rate;
}
</code></pre>



</details>

<a name="0x0_Libra_lbr_exchange_rate"></a>

## Function `lbr_exchange_rate`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_lbr_exchange_rate">lbr_exchange_rate</a>&lt;CoinType&gt;(): <a href="fixedpoint32.md#0x0_FixedPoint32_T">FixedPoint32::T</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_lbr_exchange_rate">lbr_exchange_rate</a>&lt;CoinType&gt;(): <a href="fixedpoint32.md#0x0_FixedPoint32_T">FixedPoint32::T</a>
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    *&borrow_global&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="#0x0_Libra_currency_addr">currency_addr</a>()).to_lbr_exchange_rate
}
</code></pre>



</details>

<a name="0x0_Libra_update_minting_ability"></a>

## Function `update_minting_ability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_update_minting_ability">update_minting_ability</a>&lt;CoinType&gt;(can_mint: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_update_minting_ability">update_minting_ability</a>&lt;CoinType&gt;(can_mint: bool)
<b>acquires</b> <a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a> {
    <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;();
    <b>let</b> currency_info = borrow_global_mut&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(Transaction::sender());
    currency_info.can_mint = can_mint;
}
</code></pre>



</details>

<a name="0x0_Libra_currency_addr"></a>

## Function `currency_addr`



<pre><code><b>fun</b> <a href="#0x0_Libra_currency_addr">currency_addr</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Libra_currency_addr">currency_addr</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x0_Libra_assert_assoc_and_currency"></a>

## Function `assert_assoc_and_currency`



<pre><code><b>fun</b> <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Libra_assert_assoc_and_currency">assert_assoc_and_currency</a>&lt;CoinType&gt;() {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    <a href="#0x0_Libra_assert_is_coin">assert_is_coin</a>&lt;CoinType&gt;();
}
</code></pre>



</details>

<a name="0x0_Libra_assert_is_coin"></a>

## Function `assert_is_coin`



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



<pre><code>pragma verify = <b>true</b>;
</code></pre>




<a name="0x0_Libra_spec_currency_addr"></a>


<pre><code><b>define</b> <a href="#0x0_Libra_spec_currency_addr">spec_currency_addr</a>(): address { 0xA550C18 }
<a name="0x0_Libra_spec_is_currency"></a>
<b>define</b> <a href="#0x0_Libra_spec_is_currency">spec_is_currency</a>&lt;CoinType&gt;(): bool {
    exists&lt;<a href="#0x0_Libra_CurrencyInfo">CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="#0x0_Libra_spec_currency_addr">spec_currency_addr</a>())
}
</code></pre>



<a name="0x0_Libra_Specification_T"></a>

### Struct `T`


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Libra_T">T</a>&lt;CoinType&gt;
</code></pre>



<dl>
<dt>

<code>value: u64</code>
</dt>
<dd>

</dd>
</dl>



<pre><code><b>invariant</b> <b>pack</b> <a href="#0x0_Libra_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; = <a href="#0x0_Libra_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; + value;
<b>invariant</b> <b>unpack</b> <a href="#0x0_Libra_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; = <a href="#0x0_Libra_sum_of_coin_values">sum_of_coin_values</a>&lt;CoinType&gt; - value;
</code></pre>



<a name="0x0_Libra_Specification_register_currency"></a>

### Function `register_currency`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Libra_register_currency">register_currency</a>&lt;CoinType&gt;(to_lbr_exchange_rate: <a href="fixedpoint32.md#0x0_FixedPoint32_T">FixedPoint32::T</a>, is_synthetic: bool, scaling_factor: u64, fractional_part: u64, currency_code: vector&lt;u8&gt;)
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
