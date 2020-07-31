
<a name="0x1_DesignatedDealer"></a>

# Module `0x1::DesignatedDealer`

### Table of Contents

-  [Resource `Dealer`](#0x1_DesignatedDealer_Dealer)
-  [Resource `TierInfo`](#0x1_DesignatedDealer_TierInfo)
-  [Struct `ReceivedMintEvent`](#0x1_DesignatedDealer_ReceivedMintEvent)
-  [Function `publish_designated_dealer_credential`](#0x1_DesignatedDealer_publish_designated_dealer_credential)
-  [Function `add_currency`](#0x1_DesignatedDealer_add_currency)
-  [Function `add_tier`](#0x1_DesignatedDealer_add_tier)
-  [Function `update_tier`](#0x1_DesignatedDealer_update_tier)
-  [Function `tiered_mint`](#0x1_DesignatedDealer_tiered_mint)
-  [Function `exists_at`](#0x1_DesignatedDealer_exists_at)
-  [Function `validate_and_record_mint`](#0x1_DesignatedDealer_validate_and_record_mint)
-  [Function `reset_window`](#0x1_DesignatedDealer_reset_window)
-  [Specification](#0x1_DesignatedDealer_Specification)
    -  [Resource `TierInfo`](#0x1_DesignatedDealer_Specification_TierInfo)
    -  [Function `publish_designated_dealer_credential`](#0x1_DesignatedDealer_Specification_publish_designated_dealer_credential)
    -  [Function `add_currency`](#0x1_DesignatedDealer_Specification_add_currency)
    -  [Function `add_tier`](#0x1_DesignatedDealer_Specification_add_tier)
    -  [Function `update_tier`](#0x1_DesignatedDealer_Specification_update_tier)
    -  [Function `tiered_mint`](#0x1_DesignatedDealer_Specification_tiered_mint)
    -  [Function `exists_at`](#0x1_DesignatedDealer_Specification_exists_at)



<a name="0x1_DesignatedDealer_Dealer"></a>

## Resource `Dealer`

A
<code><a href="#0x1_DesignatedDealer">DesignatedDealer</a></code> always holds this
<code><a href="#0x1_DesignatedDealer_Dealer">Dealer</a></code> resource regardless of the
currencies it can hold. All
<code><a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a></code> events for all
currencies will be emitted on
<code>mint_event_handle</code>.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>mint_event_handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_DesignatedDealer_ReceivedMintEvent">DesignatedDealer::ReceivedMintEvent</a>&gt;</code>
</dt>
<dd>
 Handle for mint events
</dd>
</dl>


</details>

<a name="0x1_DesignatedDealer_TierInfo"></a>

## Resource `TierInfo`

The
<code><a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a></code> resource holds the information needed to track which
tier a mint to a DD needs to be in.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;
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

<code>tiers: vector&lt;u64&gt;</code>
</dt>
<dd>
 0-indexed array of tier upperbounds
</dd>
</dl>


</details>

<a name="0x1_DesignatedDealer_ReceivedMintEvent"></a>

## Struct `ReceivedMintEvent`

Message for mint events


<pre><code><b>struct</b> <a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 The currency minted
</dd>
<dt>

<code>destination_address: address</code>
</dt>
<dd>
 The address that receives the mint
</dd>
<dt>

<code>amount: u64</code>
</dt>
<dd>
 The amount minted (in base units of
<code>currency_code</code>)
</dd>
</dl>


</details>

<a name="0x1_DesignatedDealer_publish_designated_dealer_credential"></a>

## Function `publish_designated_dealer_credential`

Publishes a
<code><a href="#0x1_DesignatedDealer_Dealer">Dealer</a></code> resource under
<code>dd</code> with a
<code><a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a></code>,
<code>Preburn</code>, and default tiers for
<code>CoinType</code>.
If
<code>add_all_currencies = <b>true</b></code> this will add a
<code><a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a></code>,
<code>Preburn</code>,
and default tiers for each known currency at launch.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_publish_designated_dealer_credential">publish_designated_dealer_credential</a>&lt;CoinType&gt;(dd: &signer, tc_account: &signer, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_publish_designated_dealer_credential">publish_designated_dealer_credential</a>&lt;CoinType&gt;(
    dd: &signer,
    tc_account: &signer,
    add_all_currencies: bool,
) <b>acquires</b> <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(tc_account), EACCOUNT_NOT_TREASURY_COMPLIANCE);
    move_to(dd, <a href="#0x1_DesignatedDealer_Dealer">Dealer</a> { mint_event_handle: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>&gt;(dd) });
    <b>if</b> (add_all_currencies) {
        <a href="#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(dd, tc_account);
        <a href="#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(dd, tc_account);
    } <b>else</b> {
        <a href="#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;CoinType&gt;(dd, tc_account);
    };
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_add_currency"></a>

## Function `add_currency`

Adds the needed resources to the DD account
<code>dd</code> in order to work with
<code>CoinType</code>.
Public so that a currency can be added to a DD later on. Will require
multi-signer transactions in order to add a new currency to an existing DD.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;CoinType&gt;(dd: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;CoinType&gt;(dd: &signer, tc_account: &signer)
<b>acquires</b> <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <b>let</b> dd_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(dd);
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(tc_account), EACCOUNT_NOT_TREASURY_COMPLIANCE);
    <b>assert</b>(<a href="#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr), ENOT_A_DD);
    <a href="Libra.md#0x1_Libra_publish_preburn_to_account">Libra::publish_preburn_to_account</a>&lt;CoinType&gt;(dd, tc_account);
    move_to(dd, <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt; {
        window_start: <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>(),
        window_inflow: 0,
        tiers: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>(),
    });
    // Add tier amounts in base_units of CoinType
    <b>let</b> coin_scaling_factor = <a href="Libra.md#0x1_Libra_scaling_factor">Libra::scaling_factor</a>&lt;CoinType&gt;();
    <a href="#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account, dd_addr, TIER_0_DEFAULT * coin_scaling_factor);
    <a href="#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account, dd_addr, TIER_1_DEFAULT * coin_scaling_factor);
    <a href="#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account, dd_addr, TIER_2_DEFAULT * coin_scaling_factor);
    <a href="#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account, dd_addr, TIER_3_DEFAULT * coin_scaling_factor);
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_add_tier"></a>

## Function `add_tier`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account: &signer, dd_addr: address, tier_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(
    tc_account: &signer,
    dd_addr: address,
    tier_upperbound: u64
) <b>acquires</b> <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(tc_account), EACCOUNT_NOT_TREASURY_COMPLIANCE);
    <b>let</b> tiers = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers;
    <b>let</b> number_of_tiers = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(tiers);
    <b>assert</b>(number_of_tiers + 1 &lt;= MAX_NUM_TIERS, EINVALID_TIER_ADDITION);
    <b>if</b> (number_of_tiers &gt; 0) {
        <b>let</b> last_tier = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, number_of_tiers - 1);
        <b>assert</b>(last_tier &lt; tier_upperbound, EINVALID_TIER_START);
    };
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(tiers, tier_upperbound);
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_update_tier"></a>

## Function `update_tier`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_update_tier">update_tier</a>&lt;CoinType&gt;(tc_account: &signer, dd_addr: address, tier_index: u64, new_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_update_tier">update_tier</a>&lt;CoinType&gt;(
    tc_account: &signer,
    dd_addr: address,
    tier_index: u64,
    new_upperbound: u64
) <b>acquires</b> <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(tc_account), EACCOUNT_NOT_TREASURY_COMPLIANCE);
    <b>let</b> tiers = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers;
    <b>let</b> number_of_tiers = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(tiers);
    <b>assert</b>(tier_index &lt; number_of_tiers, EINVALID_TIER_INDEX);
    // Make sure that this new start for the tier is consistent with the tier above and below it.
    <b>let</b> tier = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, tier_index);
    <b>if</b> (*tier == new_upperbound) <b>return</b>;
    <b>if</b> (*tier &lt; new_upperbound) {
        <b>let</b> next_tier_index = tier_index + 1;
        <b>if</b> (next_tier_index &lt; number_of_tiers) {
            <b>assert</b>(new_upperbound &lt; *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, next_tier_index), EINVALID_TIER_START);
        };
    };
    <b>if</b> (*tier &gt; new_upperbound && tier_index &gt; 0) {
        <b>let</b> prev_tier_index = tier_index - 1;
        <b>assert</b>(new_upperbound &gt; *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, prev_tier_index), EINVALID_TIER_START);
    };
    *<a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(tiers, tier_index) = new_upperbound;
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_tiered_mint"></a>

## Function `tiered_mint`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(tc_account: &signer, amount: u64, dd_addr: address, tier_index: u64): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(
    tc_account: &signer,
    amount: u64,
    dd_addr: address,
    tier_index: u64,
): <a href="Libra.md#0x1_Libra">Libra</a>&lt;CoinType&gt; <b>acquires</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a>, <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(tc_account), EACCOUNT_NOT_TREASURY_COMPLIANCE);
    <b>assert</b>(amount &gt; 0, EINVALID_MINT_AMOUNT);
    <b>assert</b>(<a href="#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr), ENOT_A_DD);

    <a href="#0x1_DesignatedDealer_validate_and_record_mint">validate_and_record_mint</a>&lt;CoinType&gt;(dd_addr, amount, tier_index);
    // Send <a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr).mint_event_handle,
        <a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a> {
            currency_code: <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;CoinType&gt;(),
            destination_address: dd_addr,
            amount: amount,
        },
    );
    <a href="Libra.md#0x1_Libra_mint">Libra::mint</a>&lt;CoinType&gt;(tc_account, amount)
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr: address): bool {
    exists&lt;<a href="#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr)
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_validate_and_record_mint"></a>

## Function `validate_and_record_mint`

Validate and record the minting of
<code>amount</code> of
<code>CoinType</code> coins against
the
<code><a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;</code> for the DD at
<code>dd_addr</code>. Aborts if an invalid
<code>tier_index</code> is supplied, or if the inflow over the time period exceeds
that amount that can be minted according to the bounds for the
<code>tier_index</code> tier.


<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_validate_and_record_mint">validate_and_record_mint</a>&lt;CoinType&gt;(dd_addr: address, amount: u64, tier_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_validate_and_record_mint">validate_and_record_mint</a>&lt;CoinType&gt;(dd_addr: address, amount: u64, tier_index: u64)
<b>acquires</b> <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <b>let</b> tier = borrow_global_mut&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
    <a href="#0x1_DesignatedDealer_reset_window">reset_window</a>(tier);
    <b>let</b> cur_inflow = tier.window_inflow;
    <b>let</b> new_inflow = cur_inflow + amount;
    <b>let</b> tiers = &<b>mut</b> tier.tiers;
    <b>let</b> number_of_tiers = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(tiers);
    <b>assert</b>(tier_index &lt; number_of_tiers, EINVALID_TIER_INDEX);
    <b>let</b> tier_upperbound: u64 = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, tier_index);
    <b>assert</b>(new_inflow &lt;= tier_upperbound, EINVALID_AMOUNT_FOR_TIER);
    tier.window_inflow = new_inflow;
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_reset_window"></a>

## Function `reset_window`



<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_reset_window">reset_window</a>&lt;CoinType&gt;(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_TierInfo">DesignatedDealer::TierInfo</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_reset_window">reset_window</a>&lt;CoinType&gt;(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;) {
    <b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
    <b>if</b> (current_time &gt; dealer.window_start + ONE_DAY) {
        dealer.window_start = current_time;
        dealer.window_inflow = 0;
    }
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_Specification"></a>

## Specification


<a name="0x1_DesignatedDealer_Specification_TierInfo"></a>

### Resource `TierInfo`


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;
</code></pre>



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

<code>tiers: vector&lt;u64&gt;</code>
</dt>
<dd>
 0-indexed array of tier upperbounds
</dd>
</dl>



<pre><code><b>invariant</b> len(tiers) &lt;= <a href="#0x1_DesignatedDealer_SPEC_MAX_NUM_TIERS">SPEC_MAX_NUM_TIERS</a>();
<b>invariant</b> forall i in 0..len(tiers), j in 0..len(tiers) where i &lt; j: tiers[i] &lt; tiers[j];
</code></pre>



<a name="0x1_DesignatedDealer_Specification_publish_designated_dealer_credential"></a>

### Function `publish_designated_dealer_credential`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_publish_designated_dealer_credential">publish_designated_dealer_credential</a>&lt;CoinType&gt;(dd: &signer, tc_account: &signer, add_all_currencies: bool)
</code></pre>



TODO(wrwg): takes a long time but verifies.


<pre><code>pragma verify_duration_estimate = 80;
</code></pre>



<a name="0x1_DesignatedDealer_Specification_add_currency"></a>

### Function `add_currency`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;CoinType&gt;(dd: &signer, tc_account: &signer)
</code></pre>



TODO(wrwg): sort out strange behavior: verifies wo/ problem locally, but times out in Ci


<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_DesignatedDealer_Specification_add_tier"></a>

### Function `add_tier`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account: &signer, dd_addr: address, tier_upperbound: u64)
</code></pre>




<pre><code><b>ensures</b> len(<b>global</b>&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers) == len(<b>old</b>(<b>global</b>&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr)).tiers) + 1;
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers[len(<b>global</b>&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers) - 1] == tier_upperbound;
</code></pre>



<a name="0x1_DesignatedDealer_Specification_update_tier"></a>

### Function `update_tier`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_update_tier">update_tier</a>&lt;CoinType&gt;(tc_account: &signer, dd_addr: address, tier_index: u64, new_upperbound: u64)
</code></pre>




<pre><code><b>ensures</b> len(<b>global</b>&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers) == len(<b>old</b>(<b>global</b>&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr)).tiers);
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers[tier_index] == new_upperbound;
</code></pre>



<a name="0x1_DesignatedDealer_Specification_tiered_mint"></a>

### Function `tiered_mint`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(tc_account: &signer, amount: u64, dd_addr: address, tier_index: u64): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>




<a name="0x1_DesignatedDealer_dealer$11"></a>


<pre><code><b>let</b> dealer = <b>global</b>&lt;<a href="#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<a name="0x1_DesignatedDealer_current_time$12"></a>
<b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>();
<b>ensures</b> <b>old</b>(dealer.window_start) &lt;= dealer.window_start;
<b>ensures</b>
    dealer.window_start == current_time && dealer.window_inflow == amount ||
    (<b>old</b>(dealer.window_start) == dealer.window_start &&
        dealer.window_inflow == <b>old</b>(dealer.window_inflow) + amount);
<b>ensures</b> tier_index &lt; len(<b>old</b>(dealer).tiers);
<b>ensures</b> dealer.window_inflow &lt;= <b>old</b>(dealer).tiers[tier_index];
</code></pre>



<a name="0x1_DesignatedDealer_Specification_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr: address): bool
</code></pre>




<pre><code>pragma verify = <b>true</b>;
<b>ensures</b> result == <a href="#0x1_DesignatedDealer_spec_exists_at">spec_exists_at</a>(dd_addr);
</code></pre>




<a name="0x1_DesignatedDealer_spec_exists_at"></a>


<pre><code><b>define</b> <a href="#0x1_DesignatedDealer_spec_exists_at">spec_exists_at</a>(addr: address): bool {
    exists&lt;<a href="#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(addr)
}
</code></pre>




<pre><code>pragma verify = <b>true</b>;
<a name="0x1_DesignatedDealer_SPEC_MAX_NUM_TIERS"></a>
<b>define</b> <a href="#0x1_DesignatedDealer_SPEC_MAX_NUM_TIERS">SPEC_MAX_NUM_TIERS</a>(): u64 { 4 }
<a name="0x1_DesignatedDealer_spec_window_length"></a>
<b>define</b> <a href="#0x1_DesignatedDealer_spec_window_length">spec_window_length</a>(): u64 { 86400000000 }
</code></pre>
