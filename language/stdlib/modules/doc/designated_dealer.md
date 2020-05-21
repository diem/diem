
<a name="0x0_DesignatedDealer"></a>

# Module `0x0::DesignatedDealer`

### Table of Contents

-  [Struct `Dealer`](#0x0_DesignatedDealer_Dealer)
-  [Function `create_designated_dealer`](#0x0_DesignatedDealer_create_designated_dealer)
-  [Function `add_tier`](#0x0_DesignatedDealer_add_tier)
-  [Function `update_tier`](#0x0_DesignatedDealer_update_tier)
-  [Function `tiered_mint`](#0x0_DesignatedDealer_tiered_mint)
-  [Function `is_designated_dealer`](#0x0_DesignatedDealer_is_designated_dealer)
-  [Function `reset_window`](#0x0_DesignatedDealer_reset_window)
-  [Function `assert_account_is_blessed`](#0x0_DesignatedDealer_assert_account_is_blessed)
-  [Function `window_length`](#0x0_DesignatedDealer_window_length)
-  [Function `treasury_compliance_account`](#0x0_DesignatedDealer_treasury_compliance_account)



<a name="0x0_DesignatedDealer_Dealer"></a>

## Struct `Dealer`



<pre><code><b>struct</b> <a href="#0x0_DesignatedDealer_Dealer">Dealer</a>
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
 The minted inflow during this time window
</dd>
<dt>

<code>is_certified: bool</code>
</dt>
<dd>
 Association grants
</dd>
<dt>

<code>tiers: vector&lt;u64&gt;</code>
</dt>
<dd>
 0-indexed array of tier upperbounds
</dd>
</dl>


</details>

<a name="0x0_DesignatedDealer_create_designated_dealer"></a>

## Function `create_designated_dealer`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_create_designated_dealer">create_designated_dealer</a>(): <a href="#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_create_designated_dealer">create_designated_dealer</a>(
): <a href="#0x0_DesignatedDealer_Dealer">Dealer</a> {
    <a href="#0x0_DesignatedDealer_Dealer">Dealer</a> {
        window_start: <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>(),
        window_inflow: 0,
        is_certified: <b>true</b>,
        tiers: <a href="vector.md#0x0_Vector_empty">Vector::empty</a>(),
    }
}
</code></pre>



</details>

<a name="0x0_DesignatedDealer_add_tier"></a>

## Function `add_tier`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_add_tier">add_tier</a>(dealer: &<b>mut</b> <a href="#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>, next_tier_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_add_tier">add_tier</a>(dealer: &<b>mut</b> <a href="#0x0_DesignatedDealer_Dealer">Dealer</a>, next_tier_upperbound: u64)
{
    <b>let</b> tiers = &<b>mut</b> dealer.tiers;
    <b>let</b> number_of_tiers: u64 = <a href="vector.md#0x0_Vector_length">Vector::length</a>(tiers);
    // INVALID_TIER_ADDITION
    Txn::assert(number_of_tiers &lt;= 4, 3);
    <b>if</b> (number_of_tiers &gt; 1) {
        <b>let</b> prev_tier = *<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(tiers, number_of_tiers - 1);
        // INVALID_TIER_START
        Txn::assert(prev_tier &lt; next_tier_upperbound, 4);
    };
    <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(tiers, next_tier_upperbound);
}
</code></pre>



</details>

<a name="0x0_DesignatedDealer_update_tier"></a>

## Function `update_tier`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_update_tier">update_tier</a>(dealer: &<b>mut</b> <a href="#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>, tier_index: u64, new_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_update_tier">update_tier</a>(dealer: &<b>mut</b> <a href="#0x0_DesignatedDealer_Dealer">Dealer</a>, tier_index: u64, new_upperbound: u64)
{
    <b>let</b> tiers = &<b>mut</b> dealer.tiers;
    <b>let</b> number_of_tiers = <a href="vector.md#0x0_Vector_length">Vector::length</a>(tiers);
    // INVALID_TIER_INDEX
    Txn::assert(tier_index &lt;= 4, 3);
    Txn::assert(tier_index &lt; number_of_tiers, 3);
    // Make sure that this new start for the tier is consistent
    // with the tier above it.
    <b>let</b> next_tier = tier_index + 1;
    <b>if</b> (next_tier &lt; number_of_tiers) {
        // INVALID_TIER_START
        Txn::assert(new_upperbound &lt; *<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(tiers, next_tier), 4);
    };
    <b>let</b> tier_mut = <a href="vector.md#0x0_Vector_borrow_mut">Vector::borrow_mut</a>(tiers, tier_index);
    *tier_mut = new_upperbound;
}
</code></pre>



</details>

<a name="0x0_DesignatedDealer_tiered_mint"></a>

## Function `tiered_mint`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_tiered_mint">tiered_mint</a>(dealer: &<b>mut</b> <a href="#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>, amount: u64, tier_index: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_tiered_mint">tiered_mint</a>(dealer: &<b>mut</b> <a href="#0x0_DesignatedDealer_Dealer">Dealer</a>, amount: u64, tier_index: u64): bool {
    <a href="#0x0_DesignatedDealer_reset_window">reset_window</a>(dealer);
    <b>let</b> cur_inflow = *&dealer.window_inflow;
    <b>let</b> tiers = &<b>mut</b> dealer.tiers;
    // If the tier_index is one past the bounded tiers, minting is unbounded
    <b>let</b> number_of_tiers = <a href="vector.md#0x0_Vector_length">Vector::length</a>(tiers);
    <b>let</b> tier_check = &<b>mut</b> <b>false</b>;
    <b>if</b> (tier_index == number_of_tiers) {
        *tier_check = <b>true</b>;
    } <b>else</b> {
        <b>let</b> tier_upperbound: u64 = *<a href="vector.md#0x0_Vector_borrow">Vector::borrow</a>(tiers, tier_index);
        *tier_check = (cur_inflow + amount &lt;= tier_upperbound);
    };
    <b>if</b> (*tier_check) {
        dealer.window_inflow = cur_inflow + amount;
    };
    *tier_check
}
</code></pre>



</details>

<a name="0x0_DesignatedDealer_is_designated_dealer"></a>

## Function `is_designated_dealer`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_is_designated_dealer">is_designated_dealer</a>(dealer: &<a href="#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_is_designated_dealer">is_designated_dealer</a>(dealer: &<a href="#0x0_DesignatedDealer_Dealer">Dealer</a>): bool
{
    *&dealer.is_certified
}
</code></pre>



</details>

<a name="0x0_DesignatedDealer_reset_window"></a>

## Function `reset_window`



<pre><code><b>fun</b> <a href="#0x0_DesignatedDealer_reset_window">reset_window</a>(dealer: &<b>mut</b> <a href="#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_DesignatedDealer_reset_window">reset_window</a>(dealer: &<b>mut</b> <a href="#0x0_DesignatedDealer_Dealer">Dealer</a>) {
    <b>let</b> current_time = <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
    <b>if</b> (current_time &gt; dealer.window_start + <a href="#0x0_DesignatedDealer_window_length">window_length</a>()) {
        dealer.window_start = current_time;
        dealer.window_inflow = 0;
    }
}
</code></pre>



</details>

<a name="0x0_DesignatedDealer_assert_account_is_blessed"></a>

## Function `assert_account_is_blessed`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_assert_account_is_blessed">assert_account_is_blessed</a>(sender_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_DesignatedDealer_assert_account_is_blessed">assert_account_is_blessed</a>(sender_account: &signer) {
    // Verify that the sender is treasury compliant account
    Txn::assert(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(sender_account) == <a href="#0x0_DesignatedDealer_treasury_compliance_account">treasury_compliance_account</a>(), 0)
}
</code></pre>



</details>

<a name="0x0_DesignatedDealer_window_length"></a>

## Function `window_length`



<pre><code><b>fun</b> <a href="#0x0_DesignatedDealer_window_length">window_length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_DesignatedDealer_window_length">window_length</a>(): u64 {
    // number of microseconds in a day
    86400000000
}
</code></pre>



</details>

<a name="0x0_DesignatedDealer_treasury_compliance_account"></a>

## Function `treasury_compliance_account`



<pre><code><b>fun</b> <a href="#0x0_DesignatedDealer_treasury_compliance_account">treasury_compliance_account</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_DesignatedDealer_treasury_compliance_account">treasury_compliance_account</a>(): address {
    0xB1E55ED
}
</code></pre>



</details>
