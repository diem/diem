
<a name="0x0_AccountLimits"></a>

# Module `0x0::AccountLimits`

### Table of Contents

-  [Struct `UpdateCapability`](#0x0_AccountLimits_UpdateCapability)
-  [Struct `LimitsDefinition`](#0x0_AccountLimits_LimitsDefinition)
-  [Struct `Window`](#0x0_AccountLimits_Window)
-  [Function `grant_account_tracking`](#0x0_AccountLimits_grant_account_tracking)
-  [Function `update_deposit_limits`](#0x0_AccountLimits_update_deposit_limits)
-  [Function `update_withdrawal_limits`](#0x0_AccountLimits_update_withdrawal_limits)
-  [Function `create`](#0x0_AccountLimits_create)
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



<a name="0x0_AccountLimits_UpdateCapability"></a>

## Struct `UpdateCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountLimits_UpdateCapability">UpdateCapability</a>
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



<pre><code><b>struct</b> <a href="#0x0_AccountLimits_Window">Window</a>
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
</dl>


</details>

<a name="0x0_AccountLimits_grant_account_tracking"></a>

## Function `grant_account_tracking`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_grant_account_tracking">grant_account_tracking</a>(): <a href="#0x0_AccountLimits_UpdateCapability">AccountLimits::UpdateCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_grant_account_tracking">grant_account_tracking</a>(): <a href="#0x0_AccountLimits_UpdateCapability">UpdateCapability</a> {
    // This address needs <b>to</b> match the singleton_addr in <a href="account_tracking.md#0x0_AccountTrack">AccountTrack</a>
    Transaction::assert(Transaction::sender() == 0xA550C18, 2);
    <a href="#0x0_AccountLimits_UpdateCapability">UpdateCapability</a>{}
}
</code></pre>



</details>

<a name="0x0_AccountLimits_update_deposit_limits"></a>

## Function `update_deposit_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(amount: u64, receiving_limits_addr: address, receiving_window_info: &<b>mut</b> <a href="#0x0_AccountLimits_Window">AccountLimits::Window</a>, _cap: &<a href="#0x0_AccountLimits_UpdateCapability">AccountLimits::UpdateCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(
    amount: u64,
    receiving_limits_addr: address,
    receiving_window_info: &<b>mut</b> <a href="#0x0_AccountLimits_Window">Window</a>,
    _cap: &<a href="#0x0_AccountLimits_UpdateCapability">UpdateCapability</a>
): bool <b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">0x0::Testnet::is_testnet</a>(), 10047);
    <a href="#0x0_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(
        amount,
        receiving_window_info,
        borrow_global&lt;<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(receiving_limits_addr),
    )
}
</code></pre>



</details>

<a name="0x0_AccountLimits_update_withdrawal_limits"></a>

## Function `update_withdrawal_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(amount: u64, limits_addr: address, account_window_info: &<b>mut</b> <a href="#0x0_AccountLimits_Window">AccountLimits::Window</a>, _cap: &<a href="#0x0_AccountLimits_UpdateCapability">AccountLimits::UpdateCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(
    amount: u64,
    limits_addr: address,
    account_window_info: &<b>mut</b> <a href="#0x0_AccountLimits_Window">Window</a>,
    _cap: &<a href="#0x0_AccountLimits_UpdateCapability">UpdateCapability</a>
): bool <b>acquires</b> <a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">0x0::Testnet::is_testnet</a>(), 10048);
    <a href="#0x0_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(
        amount,
        account_window_info,
        borrow_global&lt;<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(limits_addr),
    )
}
</code></pre>



</details>

<a name="0x0_AccountLimits_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_create">create</a>(): <a href="#0x0_AccountLimits_Window">AccountLimits::Window</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountLimits_create">create</a>(): <a href="#0x0_AccountLimits_Window">Window</a> {
    <a href="#0x0_AccountLimits_Window">Window</a> {
        window_start: <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>(),
        window_outflow: 0,
        window_inflow: 0,
        tracked_balance: 0,
    }
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



<pre><code><b>fun</b> <a href="#0x0_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(amount: u64, receiving: &<b>mut</b> <a href="#0x0_AccountLimits_Window">AccountLimits::Window</a>, limits_definition: &<a href="#0x0_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(
    amount: u64,
    receiving: &<b>mut</b> <a href="#0x0_AccountLimits_Window">Window</a>,
    limits_definition: &<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>
): bool {
    Transaction::assert(limits_definition.is_certified, 1);
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



<pre><code><b>fun</b> <a href="#0x0_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(amount: u64, sending: &<b>mut</b> <a href="#0x0_AccountLimits_Window">AccountLimits::Window</a>, limits_definition: &<a href="#0x0_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(
    amount: u64,
    sending: &<b>mut</b> <a href="#0x0_AccountLimits_Window">Window</a>,
    limits_definition: &<a href="#0x0_AccountLimits_LimitsDefinition">LimitsDefinition</a>
): bool {
    Transaction::assert(limits_definition.is_certified, 1);
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
