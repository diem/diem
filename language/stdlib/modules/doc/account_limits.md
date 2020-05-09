
<a name="0x0_AccountLimits"></a>

# Module `0x0::AccountLimits`

### Table of Contents

-  [Struct `CallingCapability`](#0x0_AccountLimits_CallingCapability)
-  [Struct `LimitsDefinition`](#0x0_AccountLimits_LimitsDefinition)
-  [Struct `Window`](#0x0_AccountLimits_Window)
-  [Function `grant_calling_capability`](#0x0_AccountLimits_grant_calling_capability)
-  [Function `update_deposit_limits`](#0x0_AccountLimits_update_deposit_limits)
-  [Function `update_withdrawal_limits`](#0x0_AccountLimits_update_withdrawal_limits)
-  [Function `publish`](#0x0_AccountLimits_publish)
-  [Function `publish_limits_definition`](#0x0_AccountLimits_publish_limits_definition)
-  [Function `publish_unrestricted_limits`](#0x0_AccountLimits_publish_unrestricted_limits)
-  [Function `unpublish_limits_definition`](#0x0_AccountLimits_unpublish_limits_definition)
-  [Function `certify_limits_definition`](#0x0_AccountLimits_certify_limits_definition)
-  [Function `decertify_limits_definition`](#0x0_AccountLimits_decertify_limits_definition)
-  [Function `default_limits_addr`](#0x0_AccountLimits_default_limits_addr)
-  [Function `reset_window`](#0x0_AccountLimits_reset_window)
-  [Function `can_receive`](#0x0_AccountLimits_can_receive)
-  [Function `can_withdraw`](#0x0_AccountLimits_can_withdraw)
-  [Function `is_unrestricted`](#0x0_AccountLimits_is_unrestricted)
-  [Function `limits_definition_address`](#0x0_AccountLimits_limits_definition_address)
-  [Function `is_unlimited_account`](#0x0_AccountLimits_is_unlimited_account)
-  [Function `current_time`](#0x0_AccountLimits_current_time)



<a name="0x0_AccountLimits_CallingCapability"></a>

## Struct `CallingCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountLimits_CallingCapability">CallingCapability</a>
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

<a name="0x0_AccountLimits_LimitsDefinition"></a>

## Struct `LimitsDefinition`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>max_outflow: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>max_inflow: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>time_period: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>max_holding: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>is_certified: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_AccountLimits_Window"></a>

## Struct `Window`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountLimits_Window">Window</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>window_start: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>window_outflow: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>window_inflow: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>tracked_balance: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>limits_definition: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_AccountLimits_grant_calling_capability"></a>

## Function `grant_calling_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_grant_calling_capability">grant_calling_capability</a>(): <a href="#0x0_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_grant_calling_capability">grant_calling_capability</a>(): <a href="#0x0_AccountLimits_CallingCapability">CallingCapability</a> {
    Transaction::assert(Transaction::sender() == 0xA550C18, 3000);
    <a href="#0x0_AccountLimits_CallingCapability">CallingCapability</a>{}
}
</code></pre>



</details>

<a name="0x0_AccountLimits_update_deposit_limits"></a>

## Function `update_deposit_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(amount: u64, addr: address, _cap: &<a href="#0x0_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(
    amount: u64,
    addr: address,
    _cap: &<a href="#0x0_AccountLimits_CallingCapability">CallingCapability</a>,
): bool <b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>, <a href="#0x0_AccountLimits_Window">Window</a> {
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">0x0::Testnet::is_testnet</a>(), 10047);
    <a href="#0x0_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(
        amount,
        borrow_global_mut&lt;<a href="#0x0_AccountLimits_Window">Window</a>&gt;(addr),
    )
}
</code></pre>



</details>

<a name="0x0_AccountLimits_update_withdrawal_limits"></a>

## Function `update_withdrawal_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(amount: u64, addr: address, _cap: &<a href="#0x0_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(
    amount: u64,
    addr: address,
    _cap: &<a href="#0x0_AccountLimits_CallingCapability">CallingCapability</a>,
): bool <b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>, <a href="#0x0_AccountLimits_Window">Window</a> {
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">0x0::Testnet::is_testnet</a>(), 10048);
    <a href="#0x0_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(
        amount,
        borrow_global_mut&lt;<a href="#0x0_AccountLimits_Window">Window</a>&gt;(addr),
    )
}
</code></pre>



</details>

<a name="0x0_AccountLimits_publish"></a>

## Function `publish`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_publish">publish</a>(to_limit: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_publish">publish</a>(to_limit: &signer) {
    move_to(
        to_limit,
        <a href="#0x0_AccountLimits_Window">Window</a> {
            window_start: <a href="#0x0_AccountLimits_current_time">current_time</a>(),
            window_outflow: 0,
            window_inflow: 0,
            tracked_balance: 0,
            limits_definition: <a href="#0x0_AccountLimits_default_limits_addr">default_limits_addr</a>()
        }
    )
}
</code></pre>



</details>

<a name="0x0_AccountLimits_publish_limits_definition"></a>

## Function `publish_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_publish_limits_definition">publish_limits_definition</a>(max_outflow: u64, max_inflow: u64, max_holding: u64, time_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_publish_limits_definition">publish_limits_definition</a>(
    max_outflow: u64,
    max_inflow: u64,
    max_holding: u64,
    time_period: u64
) {
    move_to_sender(<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
        max_outflow,
        max_inflow,
        max_holding,
        time_period,
        is_certified: <b>false</b>,
    });
}
</code></pre>



</details>

<a name="0x0_AccountLimits_publish_unrestricted_limits"></a>

## Function `publish_unrestricted_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_publish_unrestricted_limits">publish_unrestricted_limits</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_publish_unrestricted_limits">publish_unrestricted_limits</a>() {
    <b>let</b> u64_max = 18446744073709551615u64;
    <a href="#0x0_AccountLimits_publish_limits_definition">publish_limits_definition</a>(u64_max, u64_max, u64_max, u64_max)
}
</code></pre>



</details>

<a name="0x0_AccountLimits_unpublish_limits_definition"></a>

## Function `unpublish_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_unpublish_limits_definition">unpublish_limits_definition</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_unpublish_limits_definition">unpublish_limits_definition</a>()
<b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
        max_outflow: _,
        max_inflow: _,
        max_holding: _,
        time_period: _,
        is_certified: _,
    } = move_from&lt;<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(Transaction::sender());
}
</code></pre>



</details>

<a name="0x0_AccountLimits_certify_limits_definition"></a>

## Function `certify_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_certify_limits_definition">certify_limits_definition</a>(limits_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_certify_limits_definition">certify_limits_definition</a>(limits_addr: address)
<b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    borrow_global_mut&lt;<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(limits_addr).is_certified = <b>true</b>;
}
</code></pre>



</details>

<a name="0x0_AccountLimits_decertify_limits_definition"></a>

## Function `decertify_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_decertify_limits_definition">decertify_limits_definition</a>(limits_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_decertify_limits_definition">decertify_limits_definition</a>(limits_addr: address)
<b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    borrow_global_mut&lt;<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(limits_addr).is_certified = <b>false</b>;
}
</code></pre>



</details>

<a name="0x0_AccountLimits_default_limits_addr"></a>

## Function `default_limits_addr`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_default_limits_addr">default_limits_addr</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_default_limits_addr">default_limits_addr</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x0_AccountLimits_reset_window"></a>

## Function `reset_window`



<pre><code><b>fun</b> <a href="#0x0_AccountLimits_reset_window">reset_window</a>(window: &<b>mut</b> <a href="#0x0_AccountLimits_Window">AccountLimits::Window</a>, limits_definition: &<a href="#0x0_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountLimits_reset_window">reset_window</a>(window: &<b>mut</b> <a href="#0x0_AccountLimits_Window">Window</a>, limits_definition: &<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>) {
    <b>let</b> current_time = <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
    <b>if</b> (current_time &gt; window.window_start + limits_definition.time_period) {
        window.window_start = current_time;
        window.window_inflow = 0;
        window.window_outflow = 0;
    }
}
</code></pre>



</details>

<a name="0x0_AccountLimits_can_receive"></a>

## Function `can_receive`



<pre><code><b>fun</b> <a href="#0x0_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(amount: u64, receiving: &<b>mut</b> <a href="#0x0_AccountLimits_Window">AccountLimits::Window</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(
    amount: u64,
    receiving: &<b>mut</b> <a href="#0x0_AccountLimits_Window">Window</a>,
): bool <b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <b>let</b> limits_definition = borrow_global_mut&lt;<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(receiving.limits_definition);
    // If the limits ares unrestricted then no more work needs <b>to</b> be done
    <b>if</b> (<a href="#0x0_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_definition)) <b>return</b> <b>true</b>;

    <a href="#0x0_AccountLimits_reset_window">reset_window</a>(receiving, limits_definition);
    // Check that the max inflow is OK
    <b>let</b> inflow_ok = receiving.window_inflow + amount &lt;= limits_definition.max_inflow;
    // Check that the holding after the deposit is OK
    <b>let</b> holding_ok = receiving.tracked_balance + amount &lt;= limits_definition.max_holding;
    // The account with `receiving` window can receive the payment so record it.
    <b>if</b> (inflow_ok && holding_ok) {
        receiving.window_inflow = receiving.window_inflow + amount;
        receiving.tracked_balance = receiving.tracked_balance + amount;
    };
    inflow_ok && holding_ok
}
</code></pre>



</details>

<a name="0x0_AccountLimits_can_withdraw"></a>

## Function `can_withdraw`



<pre><code><b>fun</b> <a href="#0x0_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(amount: u64, sending: &<b>mut</b> <a href="#0x0_AccountLimits_Window">AccountLimits::Window</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(
    amount: u64,
    sending: &<b>mut</b> <a href="#0x0_AccountLimits_Window">Window</a>,
): bool <b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <b>let</b> limits_definition = borrow_global_mut&lt;<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(sending.limits_definition);
    // If the limits are unrestricted then no more work is required
    <b>if</b> (<a href="#0x0_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_definition)) <b>return</b> <b>true</b>;

    <a href="#0x0_AccountLimits_reset_window">reset_window</a>(sending, limits_definition);
    // Check max outlflow
    <b>let</b> outflow = sending.window_outflow + amount;
    <b>let</b> outflow_ok = outflow &lt;= limits_definition.max_outflow;
    // Outflow is OK, so record it.
    <b>if</b> (outflow_ok) {
        sending.window_outflow = outflow;
        sending.tracked_balance = <b>if</b> (amount &gt;= sending.tracked_balance) 0
                                   <b>else</b> sending.tracked_balance - amount;
    };
    outflow_ok
}
</code></pre>



</details>

<a name="0x0_AccountLimits_is_unrestricted"></a>

## Function `is_unrestricted`



<pre><code><b>fun</b> <a href="#0x0_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_def: &<a href="#0x0_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_def: &<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>): bool {
    <b>let</b> u64_max = 18446744073709551615u64;
    limits_def.max_inflow == u64_max &&
    limits_def.max_outflow == u64_max &&
    limits_def.max_holding == u64_max &&
    limits_def.time_period == u64_max
}
</code></pre>



</details>

<a name="0x0_AccountLimits_limits_definition_address"></a>

## Function `limits_definition_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_limits_definition_address">limits_definition_address</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_limits_definition_address">limits_definition_address</a>(addr: address): address <b>acquires</b> <a href="#0x0_AccountLimits_Window">Window</a> {
    borrow_global&lt;<a href="#0x0_AccountLimits_Window">Window</a>&gt;(addr).limits_definition
}
</code></pre>



</details>

<a name="0x0_AccountLimits_is_unlimited_account"></a>

## Function `is_unlimited_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_is_unlimited_account">is_unlimited_account</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_is_unlimited_account">is_unlimited_account</a>(addr: address): bool <b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <a href="#0x0_AccountLimits_is_unrestricted">is_unrestricted</a>(borrow_global&lt;<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(addr))
}
</code></pre>



</details>

<a name="0x0_AccountLimits_current_time"></a>

## Function `current_time`



<pre><code><b>fun</b> <a href="#0x0_AccountLimits_current_time">current_time</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountLimits_current_time">current_time</a>(): u64 {
    <b>if</b> (<a href="libra_time.md#0x0_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>()) 0 <b>else</b> <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>()
}
</code></pre>



</details>
