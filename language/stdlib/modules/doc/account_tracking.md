
<a name="0x0_AccountTrack"></a>

# Module `0x0::AccountTrack`

### Table of Contents

-  [Struct `AccountLimitsCapability`](#0x0_AccountTrack_AccountLimitsCapability)
-  [Struct `CallingCapability`](#0x0_AccountTrack_CallingCapability)
-  [Function `initialize`](#0x0_AccountTrack_initialize)
-  [Function `grant_calling_capability`](#0x0_AccountTrack_grant_calling_capability)
-  [Function `update_deposit_limits`](#0x0_AccountTrack_update_deposit_limits)
-  [Function `update_withdrawal_limits`](#0x0_AccountTrack_update_withdrawal_limits)
-  [Function `update_info`](#0x0_AccountTrack_update_info)
-  [Function `tracking_info`](#0x0_AccountTrack_tracking_info)
-  [Function `is_unlimited_account`](#0x0_AccountTrack_is_unlimited_account)
-  [Function `singleton_addr`](#0x0_AccountTrack_singleton_addr)



<a name="0x0_AccountTrack_AccountLimitsCapability"></a>

## Struct `AccountLimitsCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountTrack_AccountLimitsCapability">AccountLimitsCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>account_limits_cap: <a href="account_limits.md#0x0_AccountLimits_UpdateCapability">AccountLimits::UpdateCapability</a></code>
</dt>
<dd>

</dd>
<dt>

<code>update_cap: <a href="account_type.md#0x0_AccountType_UpdateCapability">AccountType::UpdateCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_AccountTrack_CallingCapability"></a>

## Struct `CallingCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_AccountTrack_CallingCapability">CallingCapability</a>
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

<a name="0x0_AccountTrack_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountTrack_initialize">initialize</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountTrack_initialize">initialize</a>() {
    Transaction::assert(Transaction::sender() == <a href="#0x0_AccountTrack_singleton_addr">singleton_addr</a>(), 3000);
    <b>let</b> account_limits_cap = <a href="account_limits.md#0x0_AccountLimits_grant_account_tracking">AccountLimits::grant_account_tracking</a>();
    <b>let</b> update_cap = <a href="account_type.md#0x0_AccountType_grant_account_tracking">AccountType::grant_account_tracking</a>();
    move_to_sender(<a href="#0x0_AccountTrack_AccountLimitsCapability">AccountLimitsCapability</a> {
        account_limits_cap, update_cap
    });
}
</code></pre>



</details>

<a name="0x0_AccountTrack_grant_calling_capability"></a>

## Function `grant_calling_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountTrack_grant_calling_capability">grant_calling_capability</a>(): <a href="#0x0_AccountTrack_CallingCapability">AccountTrack::CallingCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountTrack_grant_calling_capability">grant_calling_capability</a>(): <a href="#0x0_AccountTrack_CallingCapability">CallingCapability</a> {
    Transaction::assert(Transaction::sender() == 0xA550C18, 3000);
    <a href="#0x0_AccountTrack_CallingCapability">CallingCapability</a>{}
}
</code></pre>



</details>

<a name="0x0_AccountTrack_update_deposit_limits"></a>

## Function `update_deposit_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountTrack_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(amount: u64, receiving_addr: address, _cap: &<a href="#0x0_AccountTrack_CallingCapability">AccountTrack::CallingCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountTrack_update_deposit_limits">update_deposit_limits</a>&lt;CoinType&gt;(
    amount: u64,
    receiving_addr: address,
    _cap: &<a href="#0x0_AccountTrack_CallingCapability">CallingCapability</a>,
): bool <b>acquires</b> <a href="#0x0_AccountTrack_AccountLimitsCapability">AccountLimitsCapability</a> {
    <b>if</b> (<a href="#0x0_AccountTrack_is_unlimited_account">is_unlimited_account</a>(receiving_addr)) <b>return</b> <b>true</b>;
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">0x0::Testnet::is_testnet</a>(), 10043);
    <b>let</b> (receiving_limit_addr, receiving_info) = <a href="#0x0_AccountTrack_tracking_info">tracking_info</a>(receiving_addr);
    <b>let</b> can_send = <a href="account_limits.md#0x0_AccountLimits_update_deposit_limits">AccountLimits::update_deposit_limits</a>&lt;CoinType&gt;(
        amount,
        receiving_limit_addr,
        &<b>mut</b> receiving_info,
        &borrow_global&lt;<a href="#0x0_AccountTrack_AccountLimitsCapability">AccountLimitsCapability</a>&gt;(<a href="#0x0_AccountTrack_singleton_addr">singleton_addr</a>()).account_limits_cap
    );
    <a href="#0x0_AccountTrack_update_info">update_info</a>(receiving_addr, receiving_info);
    can_send
}
</code></pre>



</details>

<a name="0x0_AccountTrack_update_withdrawal_limits"></a>

## Function `update_withdrawal_limits`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountTrack_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(amount: u64, addr: address, _cap: &<a href="#0x0_AccountTrack_CallingCapability">AccountTrack::CallingCapability</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_AccountTrack_update_withdrawal_limits">update_withdrawal_limits</a>&lt;CoinType&gt;(
    amount: u64,
    addr: address,
    _cap: &<a href="#0x0_AccountTrack_CallingCapability">CallingCapability</a>,
): bool <b>acquires</b> <a href="#0x0_AccountTrack_AccountLimitsCapability">AccountLimitsCapability</a> {
    <b>if</b> (<a href="#0x0_AccountTrack_is_unlimited_account">is_unlimited_account</a>(addr)) <b>return</b> <b>true</b>;
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">0x0::Testnet::is_testnet</a>(), 10044);
    <b>let</b> (limits_addr, account_metadata) = <a href="#0x0_AccountTrack_tracking_info">tracking_info</a>(addr);
    <b>let</b> can_withdraw = <a href="account_limits.md#0x0_AccountLimits_update_withdrawal_limits">AccountLimits::update_withdrawal_limits</a>&lt;CoinType&gt;(
        amount,
        limits_addr,
        &<b>mut</b> account_metadata,
        &borrow_global&lt;<a href="#0x0_AccountTrack_AccountLimitsCapability">AccountLimitsCapability</a>&gt;(<a href="#0x0_AccountTrack_singleton_addr">singleton_addr</a>()).account_limits_cap
    );
    <a href="#0x0_AccountTrack_update_info">update_info</a>(addr, account_metadata);
    can_withdraw
}
</code></pre>



</details>

<a name="0x0_AccountTrack_update_info"></a>

## Function `update_info`



<pre><code><b>fun</b> <a href="#0x0_AccountTrack_update_info">update_info</a>(addr: address, info: <a href="account_limits.md#0x0_AccountLimits_Window">AccountLimits::Window</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountTrack_update_info">update_info</a>(addr: address, info: <a href="account_limits.md#0x0_AccountLimits_Window">AccountLimits::Window</a>)
<b>acquires</b> <a href="#0x0_AccountTrack_AccountLimitsCapability">AccountLimitsCapability</a> {
    <b>if</b> (<a href="account_type.md#0x0_AccountType_is_a">AccountType::is_a</a>&lt;<a href="unhosted.md#0x0_Unhosted_T">Unhosted::T</a>&gt;(addr)) {
        <b>let</b> unhosted_info = <a href="unhosted.md#0x0_Unhosted_update_account_limits">Unhosted::update_account_limits</a>(info);
        <a href="account_type.md#0x0_AccountType_update_with_capability">AccountType::update_with_capability</a>&lt;<a href="unhosted.md#0x0_Unhosted_T">Unhosted::T</a>&gt;(
            addr,
            unhosted_info,
            &borrow_global&lt;<a href="#0x0_AccountTrack_AccountLimitsCapability">AccountLimitsCapability</a>&gt;(<a href="#0x0_AccountTrack_singleton_addr">singleton_addr</a>()).update_cap
        )
    } <b>else</b> <b>if</b> (<a href="vasp.md#0x0_VASP_is_vasp">VASP::is_vasp</a>(addr)) {
        <b>abort</b> 3003
    } <b>else</b> <b>if</b> (<a href="account_type.md#0x0_AccountType_is_a">AccountType::is_a</a>&lt;<a href="empty.md#0x0_Empty_T">Empty::T</a>&gt;(addr)) {
        <b>abort</b> 3002
    } <b>else</b> { // Can add more logic here <b>as</b> we add more account types
        <b>abort</b> 3001
    }
}
</code></pre>



</details>

<a name="0x0_AccountTrack_tracking_info"></a>

## Function `tracking_info`



<pre><code><b>fun</b> <a href="#0x0_AccountTrack_tracking_info">tracking_info</a>(addr: address): (address, <a href="account_limits.md#0x0_AccountLimits_Window">AccountLimits::Window</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountTrack_tracking_info">tracking_info</a>(addr: address): (address, <a href="account_limits.md#0x0_AccountLimits_Window">AccountLimits::Window</a>) {
    <b>if</b> (<a href="account_type.md#0x0_AccountType_is_a">AccountType::is_a</a>&lt;<a href="unhosted.md#0x0_Unhosted_T">Unhosted::T</a>&gt;(addr)) {
        (
            <a href="unhosted.md#0x0_Unhosted_limits_addr">Unhosted::limits_addr</a>(),
            <a href="unhosted.md#0x0_Unhosted_account_limits">Unhosted::account_limits</a>(<a href="account_type.md#0x0_AccountType_account_metadata">AccountType::account_metadata</a>&lt;<a href="unhosted.md#0x0_Unhosted_T">Unhosted::T</a>&gt;(addr))
        )
    } <b>else</b> <b>if</b> (<a href="vasp.md#0x0_VASP_is_vasp">VASP::is_vasp</a>(addr)) {
        <b>abort</b> 3003
    } <b>else</b> <b>if</b> (<a href="account_type.md#0x0_AccountType_is_a">AccountType::is_a</a>&lt;<a href="empty.md#0x0_Empty_T">Empty::T</a>&gt;(addr)) {
        <b>abort</b> 3002
    } <b>else</b> { // Can add more logic here <b>as</b> we add more account types
        <b>abort</b> 3001
    }
}
</code></pre>



</details>

<a name="0x0_AccountTrack_is_unlimited_account"></a>

## Function `is_unlimited_account`



<pre><code><b>fun</b> <a href="#0x0_AccountTrack_is_unlimited_account">is_unlimited_account</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountTrack_is_unlimited_account">is_unlimited_account</a>(addr: address): bool {
    <a href="vasp.md#0x0_VASP_is_vasp">VASP::is_vasp</a>(addr)
}
</code></pre>



</details>

<a name="0x0_AccountTrack_singleton_addr"></a>

## Function `singleton_addr`



<pre><code><b>fun</b> <a href="#0x0_AccountTrack_singleton_addr">singleton_addr</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_AccountTrack_singleton_addr">singleton_addr</a>(): address {
    0xA550C18
}
</code></pre>



</details>
