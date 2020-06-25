
<a name="0x1_AccountLimits"></a>

# Module `0x1::AccountLimits`

### Table of Contents

-  [Resource `CallingCapability`](#0x1_AccountLimits_CallingCapability)
-  [Resource `LimitsDefinition`](#0x1_AccountLimits_LimitsDefinition)
-  [Resource `Window`](#0x1_AccountLimits_Window)
-  [Function `grant_calling_capability`](#0x1_AccountLimits_grant_calling_capability)
-  [Function `update_deposit_limits`](#0x1_AccountLimits_update_deposit_limits)
-  [Function `update_withdrawal_limits`](#0x1_AccountLimits_update_withdrawal_limits)
-  [Function `publish`](#0x1_AccountLimits_publish)
-  [Function `publish_limits_definition`](#0x1_AccountLimits_publish_limits_definition)
-  [Function `publish_unrestricted_limits`](#0x1_AccountLimits_publish_unrestricted_limits)
-  [Function `unpublish_limits_definition`](#0x1_AccountLimits_unpublish_limits_definition)
-  [Function `update_limits_definition`](#0x1_AccountLimits_update_limits_definition)
-  [Function `certify_limits_definition`](#0x1_AccountLimits_certify_limits_definition)
-  [Function `decertify_limits_definition`](#0x1_AccountLimits_decertify_limits_definition)
-  [Function `reset_window`](#0x1_AccountLimits_reset_window)
-  [Function `can_receive`](#0x1_AccountLimits_can_receive)
-  [Function `can_withdraw`](#0x1_AccountLimits_can_withdraw)
-  [Function `is_unrestricted`](#0x1_AccountLimits_is_unrestricted)
-  [Function `limits_definition_address`](#0x1_AccountLimits_limits_definition_address)
-  [Function `is_unlimited_account`](#0x1_AccountLimits_is_unlimited_account)
-  [Function `current_time`](#0x1_AccountLimits_current_time)



<a name="0x1_AccountLimits_CallingCapability"></a>

## Resource `CallingCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>
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

<a name="0x1_AccountLimits_LimitsDefinition"></a>

## Resource `LimitsDefinition`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>max_total_flow: u64</code>
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

<a name="0x1_AccountLimits_Window"></a>

## Resource `Window`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_AccountLimits_Window">Window</a>
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

<code>window_total_flow: u64</code>
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

<a name="0x1_AccountLimits_grant_calling_capability"></a>

## Function `grant_calling_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_grant_calling_capability">grant_calling_capability</a>(_: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_LibraRootRole">Roles::LibraRootRole</a>&gt;): <a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_grant_calling_capability">grant_calling_capability</a>(_: &Capability&lt;LibraRootRole&gt;): <a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a> {
    <a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>{}
}
</code></pre>



</details>

<a name="0x1_AccountLimits_update_deposit_limits"></a>

## Function `update_deposit_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(amount: u64, addr: address, _cap: &<a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(
    amount: u64,
    addr: address,
    _cap: &<a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>,
): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>, <a href="#0x1_AccountLimits_Window">Window</a> {
    <b>assert</b>(<a href="Testnet.md#0x1_Testnet_is_testnet">0x1::Testnet::is_testnet</a>(), 10047);
    <a href="#0x1_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(
        amount,
        borrow_global_mut&lt;<a href="#0x1_AccountLimits_Window">Window</a>&gt;(addr),
    )
}
</code></pre>



</details>

<a name="0x1_AccountLimits_update_withdrawal_limits"></a>

## Function `update_withdrawal_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(amount: u64, addr: address, _cap: &<a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(
    amount: u64,
    addr: address,
    _cap: &<a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>,
): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>, <a href="#0x1_AccountLimits_Window">Window</a> {
    <b>assert</b>(<a href="Testnet.md#0x1_Testnet_is_testnet">0x1::Testnet::is_testnet</a>(), 10048);
    <a href="#0x1_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(
        amount,
        borrow_global_mut&lt;<a href="#0x1_AccountLimits_Window">Window</a>&gt;(addr),
    )
}
</code></pre>



</details>

<a name="0x1_AccountLimits_publish"></a>

## Function `publish`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish">publish</a>(to_limit: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish">publish</a>(to_limit: &signer) {
    move_to(
        to_limit,
        <a href="#0x1_AccountLimits_Window">Window</a> {
            window_start: <a href="#0x1_AccountLimits_current_time">current_time</a>(),
            window_total_flow: 0,
            tracked_balance: 0,
            limits_definition: <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()
        }
    )
}
</code></pre>



</details>

<a name="0x1_AccountLimits_publish_limits_definition"></a>

## Function `publish_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_limits_definition">publish_limits_definition</a>(account: &signer, max_total_flow: u64, max_holding: u64, time_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_limits_definition">publish_limits_definition</a>(
    account: &signer,
    max_total_flow: u64,
    max_holding: u64,
    time_period: u64
) {
    move_to(
        account,
        <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
            max_total_flow,
            max_holding,
            time_period,
            is_certified: <b>false</b>,
        }
    )
}
</code></pre>



</details>

<a name="0x1_AccountLimits_publish_unrestricted_limits"></a>

## Function `publish_unrestricted_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_unrestricted_limits">publish_unrestricted_limits</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_unrestricted_limits">publish_unrestricted_limits</a>(account: &signer) {
    <b>let</b> u64_max = 18446744073709551615u64;
    <a href="#0x1_AccountLimits_publish_limits_definition">publish_limits_definition</a>(account, u64_max, u64_max, u64_max)
}
</code></pre>



</details>

<a name="0x1_AccountLimits_unpublish_limits_definition"></a>

## Function `unpublish_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_unpublish_limits_definition">unpublish_limits_definition</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_unpublish_limits_definition">unpublish_limits_definition</a>(account: &signer)
<b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
        max_total_flow: _,
        max_holding: _,
        time_period: _,
        is_certified: _,
    } = move_from&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
}
</code></pre>



</details>

<a name="0x1_AccountLimits_update_limits_definition"></a>

## Function `update_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_limits_definition">update_limits_definition</a>(_: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;, new_max_total_flow: u64, new_max_holding_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_limits_definition">update_limits_definition</a>(
    _: &Capability&lt;TreasuryComplianceRole&gt;,
    new_max_total_flow: u64,
    new_max_holding_balance: u64,
) <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    // As we don't have Optionals for txn scripts, in update_unhosted_wallet_limits.<b>move</b>
    // we <b>use</b> 0 value <b>to</b> represent a None (ie no <b>update</b> <b>to</b> that variable)
    <b>if</b> (new_max_total_flow != 0) {
        borrow_global_mut&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()).max_total_flow = new_max_total_flow;
    };
    <b>if</b> (new_max_holding_balance != 0) {
        borrow_global_mut&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()).max_holding = new_max_holding_balance;
    };
}
</code></pre>



</details>

<a name="0x1_AccountLimits_certify_limits_definition"></a>

## Function `certify_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_certify_limits_definition">certify_limits_definition</a>(_: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;, limits_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_certify_limits_definition">certify_limits_definition</a>(_: &Capability&lt;TreasuryComplianceRole&gt;, limits_addr: address)
<b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    borrow_global_mut&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(limits_addr).is_certified = <b>true</b>;
}
</code></pre>



</details>

<a name="0x1_AccountLimits_decertify_limits_definition"></a>

## Function `decertify_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_decertify_limits_definition">decertify_limits_definition</a>(_: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;, limits_addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_decertify_limits_definition">decertify_limits_definition</a>(_: &Capability&lt;TreasuryComplianceRole&gt;, limits_addr: address)
<b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    borrow_global_mut&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(limits_addr).is_certified = <b>false</b>;
}
</code></pre>



</details>

<a name="0x1_AccountLimits_reset_window"></a>

## Function `reset_window`



<pre><code><b>fun</b> <a href="#0x1_AccountLimits_reset_window">reset_window</a>(window: &<b>mut</b> <a href="#0x1_AccountLimits_Window">AccountLimits::Window</a>, limits_definition: &<a href="#0x1_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_reset_window">reset_window</a>(window: &<b>mut</b> <a href="#0x1_AccountLimits_Window">Window</a>, limits_definition: &<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>) {
    <b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
    <b>if</b> (current_time &gt; window.window_start + limits_definition.time_period) {
        window.window_start = current_time;
        window.window_total_flow = 0;
    }
}
</code></pre>



</details>

<a name="0x1_AccountLimits_can_receive"></a>

## Function `can_receive`



<pre><code><b>fun</b> <a href="#0x1_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(amount: u64, receiving: &<b>mut</b> <a href="#0x1_AccountLimits_Window">AccountLimits::Window</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(
    amount: u64,
    receiving: &<b>mut</b> <a href="#0x1_AccountLimits_Window">Window</a>,
): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <b>let</b> limits_definition = borrow_global_mut&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(receiving.limits_definition);
    // If the limits ares unrestricted then no more work needs <b>to</b> be done
    <b>if</b> (<a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_definition)) <b>return</b> <b>true</b>;

    <a href="#0x1_AccountLimits_reset_window">reset_window</a>(receiving, limits_definition);
    // Check that the max total flow is OK
    <b>let</b> total_flow_ok = receiving.window_total_flow + amount &lt;= limits_definition.max_total_flow;
    // Check that the holding after the deposit is OK
    <b>let</b> holding_ok = receiving.tracked_balance + amount &lt;= limits_definition.max_holding;
    // The account with `receiving` window can receive the payment so record it.
    <b>if</b> (total_flow_ok && holding_ok) {
        receiving.window_total_flow = receiving.window_total_flow + amount;
        receiving.tracked_balance = receiving.tracked_balance + amount;
    };
    total_flow_ok && holding_ok
}
</code></pre>



</details>

<a name="0x1_AccountLimits_can_withdraw"></a>

## Function `can_withdraw`



<pre><code><b>fun</b> <a href="#0x1_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(amount: u64, sending: &<b>mut</b> <a href="#0x1_AccountLimits_Window">AccountLimits::Window</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(
    amount: u64,
    sending: &<b>mut</b> <a href="#0x1_AccountLimits_Window">Window</a>,
): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <b>let</b> limits_definition = borrow_global_mut&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(sending.limits_definition);
    // If the limits are unrestricted then no more work is required
    <b>if</b> (<a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_definition)) <b>return</b> <b>true</b>;

    <a href="#0x1_AccountLimits_reset_window">reset_window</a>(sending, limits_definition);
    // Check total flow OK
    <b>let</b> total_flow_ok = sending.window_total_flow + amount &lt;= limits_definition.max_total_flow;
    // Flow is OK, so record it.
    <b>if</b> (total_flow_ok) {
        sending.window_total_flow = sending.window_total_flow + amount;
        sending.tracked_balance = <b>if</b> (amount &gt;= sending.tracked_balance) 0
                                   <b>else</b> sending.tracked_balance - amount;
    };
    total_flow_ok
}
</code></pre>



</details>

<a name="0x1_AccountLimits_is_unrestricted"></a>

## Function `is_unrestricted`



<pre><code><b>fun</b> <a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_def: &<a href="#0x1_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_def: &<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>): bool {
    <b>let</b> u64_max = 18446744073709551615u64;
    limits_def.max_total_flow == u64_max &&
    limits_def.max_holding == u64_max &&
    limits_def.time_period == u64_max
}
</code></pre>



</details>

<a name="0x1_AccountLimits_limits_definition_address"></a>

## Function `limits_definition_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_limits_definition_address">limits_definition_address</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_limits_definition_address">limits_definition_address</a>(addr: address): address <b>acquires</b> <a href="#0x1_AccountLimits_Window">Window</a> {
    borrow_global&lt;<a href="#0x1_AccountLimits_Window">Window</a>&gt;(addr).limits_definition
}
</code></pre>



</details>

<a name="0x1_AccountLimits_is_unlimited_account"></a>

## Function `is_unlimited_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_is_unlimited_account">is_unlimited_account</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_is_unlimited_account">is_unlimited_account</a>(addr: address): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>(borrow_global&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_AccountLimits_current_time"></a>

## Function `current_time`



<pre><code><b>fun</b> <a href="#0x1_AccountLimits_current_time">current_time</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_current_time">current_time</a>(): u64 {
    <b>if</b> (<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_not_initialized">LibraTimestamp::is_not_initialized</a>()) 0 <b>else</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>()
}
</code></pre>



</details>
