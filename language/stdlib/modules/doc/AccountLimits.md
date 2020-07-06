
<a name="0x1_AccountLimits"></a>

# Module `0x1::AccountLimits`

### Table of Contents

-  [Resource `CallingCapability`](#0x1_AccountLimits_CallingCapability)
-  [Resource `LimitsDefinition`](#0x1_AccountLimits_LimitsDefinition)
-  [Resource `Window`](#0x1_AccountLimits_Window)
-  [Function `grant_calling_capability`](#0x1_AccountLimits_grant_calling_capability)
-  [Function `initialize`](#0x1_AccountLimits_initialize)
-  [Function `update_deposit_limits`](#0x1_AccountLimits_update_deposit_limits)
-  [Function `update_withdrawal_limits`](#0x1_AccountLimits_update_withdrawal_limits)
-  [Function `publish_window`](#0x1_AccountLimits_publish_window)
-  [Function `publish_limits_definition`](#0x1_AccountLimits_publish_limits_definition)
-  [Function `publish_unrestricted_limits`](#0x1_AccountLimits_publish_unrestricted_limits)
-  [Function `update_limits_definition`](#0x1_AccountLimits_update_limits_definition)
-  [Function `set_current_holdings`](#0x1_AccountLimits_set_current_holdings)
-  [Function `reset_window`](#0x1_AccountLimits_reset_window)
-  [Function `can_receive`](#0x1_AccountLimits_can_receive)
-  [Function `can_withdraw`](#0x1_AccountLimits_can_withdraw)
-  [Function `is_unrestricted`](#0x1_AccountLimits_is_unrestricted)
-  [Function `limits_definition_address`](#0x1_AccountLimits_limits_definition_address)
-  [Function `is_unlimited_account`](#0x1_AccountLimits_is_unlimited_account)
-  [Function `has_limits_published`](#0x1_AccountLimits_has_limits_published)
-  [Function `current_time`](#0x1_AccountLimits_current_time)
-  [Specification](#0x1_AccountLimits_Specification)
    -  [Function `update_deposit_limits`](#0x1_AccountLimits_Specification_update_deposit_limits)



<a name="0x1_AccountLimits_CallingCapability"></a>

## Resource `CallingCapability`

An operations capability that restricts callers of this module since
the operations can mutate account states.


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

A resource specifying the account limits per-currency. There is a default
<code><a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a></code> resource for unhosted accounts published at
<code><a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()</code>, but other not-unhosted accounts may have
different account limit definitons. In such cases, they will have a
<code><a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a></code> published under their (root) account.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>max_inflow: u64</code>
</dt>
<dd>
 The maximum inflow allowed during the specified time period.
</dd>
<dt>

<code>max_outflow: u64</code>
</dt>
<dd>
 The maximum outflow allowed during the specified time period.
</dd>
<dt>

<code>time_period: u64</code>
</dt>
<dd>
 Time period, specified in microseconds
</dd>
<dt>

<code>max_holding: u64</code>
</dt>
<dd>
 The maximum amount that can be held
</dd>
</dl>


</details>

<a name="0x1_AccountLimits_Window"></a>

## Resource `Window`

A struct holding account transaction information for the time window
starting at
<code>window_start</code> and lasting for the
<code>time_period</code> specified
in the limits definition at
<code>limit_address</code>.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_AccountLimits_Window">Window</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>window_start: u64</code>
</dt>
<dd>
 Time window start in microseconds
</dd>
<dt>

<code>window_inflow: u64</code>
</dt>
<dd>
 The inflow during this time window
</dd>
<dt>

<code>window_outflow: u64</code>
</dt>
<dd>
 The inflow during this time window
</dd>
<dt>

<code>tracked_balance: u64</code>
</dt>
<dd>
 The balance that this account has held during this time period.
</dd>
<dt>

<code>limit_address: address</code>
</dt>
<dd>
 address storing the LimitsDefinition resource that governs this window
</dd>
</dl>


</details>

<a name="0x1_AccountLimits_grant_calling_capability"></a>

## Function `grant_calling_capability`

Grant a capability to call this module. This does not necessarily
need to be a unique capability.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_grant_calling_capability">grant_calling_capability</a>(lr_account: &signer): <a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_grant_calling_capability">grant_calling_capability</a>(lr_account: &signer): <a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a> {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), NOT_GENESIS);
    <b>assert</b>(has_libra_root_role(lr_account), NOT_LIBRA_ROOT);
    <a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>{}
}
</code></pre>



</details>

<a name="0x1_AccountLimits_initialize"></a>

## Function `initialize`

Initializes the account limits for unhosted accounts.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_initialize">initialize</a>(lr_account: &signer, calling_cap: &<a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_initialize">initialize</a>(lr_account: &signer, calling_cap: &<a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), NOT_GENESIS);
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), INVALID_INITIALIZATION_ADDRESS);
    <a href="#0x1_AccountLimits_publish_unrestricted_limits">publish_unrestricted_limits</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(lr_account, calling_cap);
    <a href="#0x1_AccountLimits_publish_unrestricted_limits">publish_unrestricted_limits</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(lr_account, calling_cap);
    <a href="#0x1_AccountLimits_publish_unrestricted_limits">publish_unrestricted_limits</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(lr_account, calling_cap);
}
</code></pre>



</details>

<a name="0x1_AccountLimits_update_deposit_limits"></a>

## Function `update_deposit_limits`

Determines if depositing
<code>amount</code> of
<code>CoinType</code> coins into the
account at
<code>addr</code> is amenable with their account limits.
Returns false if this deposit violates the account limits. Effectful.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(amount: u64, addr: address, _cap: &<a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(
    amount: u64,
    addr: address,
    _cap: &<a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>,
): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>, <a href="#0x1_AccountLimits_Window">Window</a> {
    <a href="#0x1_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(
        amount,
        borrow_global_mut&lt;<a href="#0x1_AccountLimits_Window">Window</a>&lt;CoinType&gt;&gt;(addr),
    )
}
</code></pre>



</details>

<a name="0x1_AccountLimits_update_withdrawal_limits"></a>

## Function `update_withdrawal_limits`

Determine if withdrawing
<code>amount</code> of
<code>CoinType</code> coins from
the account at
<code>addr</code> would violate the account limits for that account.
Returns
<code><b>false</b></code> if this withdrawal violates account limits. Effectful.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(amount: u64, addr: address, _cap: &<a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(
    amount: u64,
    addr: address,
    _cap: &<a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>,
): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>, <a href="#0x1_AccountLimits_Window">Window</a> {
    <a href="#0x1_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(
        amount,
        borrow_global_mut&lt;<a href="#0x1_AccountLimits_Window">Window</a>&lt;CoinType&gt;&gt;(addr),
    )
}
</code></pre>



</details>

<a name="0x1_AccountLimits_publish_window"></a>

## Function `publish_window`

All accounts that could be subject to account limits will have a
<code><a href="#0x1_AccountLimits_Window">Window</a></code> for each currency they can hold published at the top level.
Root accounts for multi-account entities will hold this resource at
their root/parent account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_window">publish_window</a>&lt;CoinType&gt;(to_limit: &signer, _: &<a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>, limit_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_window">publish_window</a>&lt;CoinType&gt;(
    to_limit: &signer,
    _: &<a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>,
    limit_address: address,
) {
    move_to(
        to_limit,
        <a href="#0x1_AccountLimits_Window">Window</a>&lt;CoinType&gt; {
            window_start: <a href="#0x1_AccountLimits_current_time">current_time</a>(),
            window_inflow: 0,
            window_outflow: 0,
            tracked_balance: 0,
            limit_address,
        }
    )
}
</code></pre>



</details>

<a name="0x1_AccountLimits_publish_limits_definition"></a>

## Function `publish_limits_definition`

Publishes a
<code><a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a></code> resource under
<code>account</code>. The caller must have permission
to publish this, represented by the
<code><a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_limits_definition">publish_limits_definition</a>&lt;CoinType&gt;(account: &signer, _: &<a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>, max_inflow: u64, max_outflow: u64, max_holding: u64, time_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_limits_definition">publish_limits_definition</a>&lt;CoinType&gt;(
    account: &signer,
    _: &<a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>,
    max_inflow: u64,
    max_outflow: u64,
    max_holding: u64,
    time_period: u64
) {
    move_to(
        account,
        <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt; {
            max_inflow,
            max_outflow,
            max_holding,
            time_period,
        }
    )
}
</code></pre>



</details>

<a name="0x1_AccountLimits_publish_unrestricted_limits"></a>

## Function `publish_unrestricted_limits`

Unrestricted accounts are represented by setting all fields in the
limits definition to u64 max.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_unrestricted_limits">publish_unrestricted_limits</a>&lt;CoinType&gt;(account: &signer, cap: &<a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_publish_unrestricted_limits">publish_unrestricted_limits</a>&lt;CoinType&gt;(account: &signer, cap: &<a href="#0x1_AccountLimits_CallingCapability">CallingCapability</a>) {
    <a href="#0x1_AccountLimits_publish_limits_definition">publish_limits_definition</a>&lt;CoinType&gt;(account, cap, U64_MAX, U64_MAX, U64_MAX, ONE_DAY)
}
</code></pre>



</details>

<a name="0x1_AccountLimits_update_limits_definition"></a>

## Function `update_limits_definition`

Updates the
<code><a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt;</code> resource at
<code>limit_address</code>.
If any of the field arguments is
<code>0</code> the corresponding field is not updated.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_limits_definition">update_limits_definition</a>&lt;CoinType&gt;(tc_account: &signer, limit_address: address, new_max_inflow: u64, new_max_outflow: u64, new_max_holding_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_limits_definition">update_limits_definition</a>&lt;CoinType&gt;(
    tc_account: &signer,
    limit_address: address,
    new_max_inflow: u64,
    new_max_outflow: u64,
    new_max_holding_balance: u64,
) <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <b>assert</b>(has_treasury_compliance_role(tc_account), NOT_TREASURY_COMPLIANCE);
    // As we don't have Optionals for txn scripts, in update_unhosted_wallet_limits.<b>move</b>
    // we <b>use</b> 0 value <b>to</b> represent a None (ie no <b>update</b> <b>to</b> that variable)
    <b>let</b> limits_def = borrow_global_mut&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt;&gt;(limit_address);
    <b>if</b> (new_max_inflow &gt; 0) { limits_def.max_inflow = new_max_inflow };
    <b>if</b> (new_max_outflow &gt; 0) { limits_def.max_outflow = new_max_outflow };
    <b>if</b> (new_max_holding_balance &gt; 0) { limits_def.max_holding = new_max_holding_balance };
}
</code></pre>



</details>

<a name="0x1_AccountLimits_set_current_holdings"></a>

## Function `set_current_holdings`

Since we don't track balances of accounts before they are limited, once
they do become limited the approximate balance in
<code>CointType</code> held by
the entity across all of its accounts will need to be set by the association.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_set_current_holdings">set_current_holdings</a>&lt;CoinType&gt;(tc_account: &signer, window_address: address, aggregate_balance: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_set_current_holdings">set_current_holdings</a>&lt;CoinType&gt;(
    tc_account: &signer,
    window_address: address,
    aggregate_balance: u64,
) <b>acquires</b> <a href="#0x1_AccountLimits_Window">Window</a> {
    <b>assert</b>(has_treasury_compliance_role(tc_account), NOT_TREASURY_COMPLIANCE);
    borrow_global_mut&lt;<a href="#0x1_AccountLimits_Window">Window</a>&lt;CoinType&gt;&gt;(window_address).tracked_balance = aggregate_balance;
}
</code></pre>



</details>

<a name="0x1_AccountLimits_reset_window"></a>

## Function `reset_window`

If the time window starting at
<code>window.window_start</code> and lasting for
<code>limits_definition.time_period</code> has elapsed, resets the window and
the inflow and outflow records.


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_reset_window">reset_window</a>&lt;CoinType&gt;(window: &<b>mut</b> <a href="#0x1_AccountLimits_Window">AccountLimits::Window</a>&lt;CoinType&gt;, limits_definition: &<a href="#0x1_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_reset_window">reset_window</a>&lt;CoinType&gt;(window: &<b>mut</b> <a href="#0x1_AccountLimits_Window">Window</a>&lt;CoinType&gt;, limits_definition: &<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt;) {
    <b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
    <b>if</b> (current_time &gt; window.window_start + limits_definition.time_period) {
        window.window_start = current_time;
        window.window_inflow = 0;
        window.window_outflow = 0;
    }
}
</code></pre>



</details>

<a name="0x1_AccountLimits_can_receive"></a>

## Function `can_receive`

Verify that the receiving account tracked by the
<code>receiving</code> window
can receive
<code>amount</code> funds without violating requirements
specified the
<code>limits_definition</code> passed in.


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(amount: u64, receiving: &<b>mut</b> <a href="#0x1_AccountLimits_Window">AccountLimits::Window</a>&lt;CoinType&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_can_receive">can_receive</a>&lt;CoinType&gt;(
    amount: u64,
    receiving: &<b>mut</b> <a href="#0x1_AccountLimits_Window">Window</a>&lt;CoinType&gt;,
): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <b>let</b> limits_definition = borrow_global_mut&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt;&gt;(receiving.limit_address);
    // If the limits are unrestricted then don't do any more work.
    <b>if</b> (<a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_definition)) <b>return</b> <b>true</b>;

    <a href="#0x1_AccountLimits_reset_window">reset_window</a>(receiving, limits_definition);
    // Check that the inflow is OK
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

<a name="0x1_AccountLimits_can_withdraw"></a>

## Function `can_withdraw`

Verify that
<code>amount</code> can be withdrawn from the account tracked
by the
<code>sending</code> window without violating any limits specified
in its
<code>limits_definition</code>.


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(amount: u64, sending: &<b>mut</b> <a href="#0x1_AccountLimits_Window">AccountLimits::Window</a>&lt;CoinType&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_can_withdraw">can_withdraw</a>&lt;CoinType&gt;(
    amount: u64,
    sending: &<b>mut</b> <a href="#0x1_AccountLimits_Window">Window</a>&lt;CoinType&gt;,
): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <b>let</b> limits_definition = borrow_global_mut&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt;&gt;(sending.limit_address);
    // If the limits are unrestricted then don't do any more work.
    <b>if</b> (<a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>(limits_definition)) <b>return</b> <b>true</b>;

    <a href="#0x1_AccountLimits_reset_window">reset_window</a>(sending, limits_definition);
    // Check outflow is OK
    <b>let</b> outflow_ok = sending.window_outflow + amount &lt;= limits_definition.max_outflow;
    // Flow is OK, so record it.
    <b>if</b> (outflow_ok) {
        sending.window_outflow = sending.window_outflow + amount;
        sending.tracked_balance = <b>if</b> (amount &gt;= sending.tracked_balance) 0
                                   <b>else</b> sending.tracked_balance - amount;
    };
    outflow_ok
}
</code></pre>



</details>

<a name="0x1_AccountLimits_is_unrestricted"></a>

## Function `is_unrestricted`

Return whether the
<code><a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a></code> resoure is unrestricted or not.


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>&lt;CoinType&gt;(limits_def: &<a href="#0x1_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>&lt;CoinType&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>&lt;CoinType&gt;(limits_def: &<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt;): bool {
    limits_def.max_inflow == U64_MAX &&
    limits_def.max_outflow == U64_MAX &&
    limits_def.max_holding == U64_MAX &&
    limits_def.time_period == ONE_DAY
}
</code></pre>



</details>

<a name="0x1_AccountLimits_limits_definition_address"></a>

## Function `limits_definition_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_limits_definition_address">limits_definition_address</a>&lt;CoinType&gt;(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_limits_definition_address">limits_definition_address</a>&lt;CoinType&gt;(addr: address): address <b>acquires</b> <a href="#0x1_AccountLimits_Window">Window</a> {
    borrow_global&lt;<a href="#0x1_AccountLimits_Window">Window</a>&lt;CoinType&gt;&gt;(addr).limit_address
}
</code></pre>



</details>

<a name="0x1_AccountLimits_is_unlimited_account"></a>

## Function `is_unlimited_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_is_unlimited_account">is_unlimited_account</a>&lt;CoinType&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_is_unlimited_account">is_unlimited_account</a>&lt;CoinType&gt;(addr: address): bool <b>acquires</b> <a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a> {
    <a href="#0x1_AccountLimits_is_unrestricted">is_unrestricted</a>(borrow_global&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt;&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_AccountLimits_has_limits_published"></a>

## Function `has_limits_published`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_has_limits_published">has_limits_published</a>&lt;CoinType&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_has_limits_published">has_limits_published</a>&lt;CoinType&gt;(addr: address): bool {
    exists&lt;<a href="#0x1_AccountLimits_LimitsDefinition">LimitsDefinition</a>&lt;CoinType&gt;&gt;(addr)
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

<a name="0x1_AccountLimits_Specification"></a>

## Specification


<a name="0x1_AccountLimits_Specification_update_deposit_limits"></a>

### Function `update_deposit_limits`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountLimits_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(amount: u64, addr: address, _cap: &<a href="#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a>): bool
</code></pre>



> TODO(wrwg): this is currently abstracted as uninterpreted function
> because of termination issue. Need to investigate why.


<pre><code>pragma verify = <b>false</b>;
pragma opaque = <b>true</b>;
<b>ensures</b> result == <a href="#0x1_AccountLimits_spec_update_deposit_limits">spec_update_deposit_limits</a>&lt;CoinType&gt;(amount, addr);
</code></pre>




<a name="0x1_AccountLimits_spec_update_deposit_limits"></a>


<pre><code><b>define</b> <a href="#0x1_AccountLimits_spec_update_deposit_limits">spec_update_deposit_limits</a>&lt;CoinType&gt;(amount: u64, addr: address): bool;
</code></pre>
