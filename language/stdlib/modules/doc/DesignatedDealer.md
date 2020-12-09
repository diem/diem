
<a name="0x1_DesignatedDealer"></a>

# Module `0x1::DesignatedDealer`

Module providing functionality for designated dealers.


-  [Resource `Dealer`](#0x1_DesignatedDealer_Dealer)
-  [Resource `TierInfo`](#0x1_DesignatedDealer_TierInfo)
-  [Struct `ReceivedMintEvent`](#0x1_DesignatedDealer_ReceivedMintEvent)
-  [Constants](#@Constants_0)
-  [Function `publish_designated_dealer_credential`](#0x1_DesignatedDealer_publish_designated_dealer_credential)
-  [Function `add_currency`](#0x1_DesignatedDealer_add_currency)
-  [Function `add_tier`](#0x1_DesignatedDealer_add_tier)
-  [Function `update_tier`](#0x1_DesignatedDealer_update_tier)
-  [Function `tiered_mint`](#0x1_DesignatedDealer_tiered_mint)
-  [Function `exists_at`](#0x1_DesignatedDealer_exists_at)
-  [Function `validate_and_record_mint`](#0x1_DesignatedDealer_validate_and_record_mint)
-  [Function `reset_window`](#0x1_DesignatedDealer_reset_window)
-  [Module Specification](#@Module_Specification_1)


<pre><code><b>use</b> <a href="Diem.md#0x1_Diem">0x1::Diem</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
<b>use</b> <a href="XUS.md#0x1_XUS">0x1::XUS</a>;
</code></pre>



<a name="0x1_DesignatedDealer_Dealer"></a>

## Resource `Dealer`

A <code><a href="DesignatedDealer.md#0x1_DesignatedDealer">DesignatedDealer</a></code> always holds this <code><a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a></code> resource regardless of the
currencies it can hold. All <code><a href="DesignatedDealer.md#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a></code> events for all
currencies will be emitted on <code>mint_event_handle</code>.


<pre><code><b>resource</b> <b>struct</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_event_handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_ReceivedMintEvent">DesignatedDealer::ReceivedMintEvent</a>&gt;</code>
</dt>
<dd>
 Handle for mint events
</dd>
</dl>


</details>

<a name="0x1_DesignatedDealer_TierInfo"></a>

## Resource `TierInfo`

The <code><a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a></code> resource holds the information needed to track which
tier a mint to a DD needs to be in.


<pre><code><b>resource</b> <b>struct</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;
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

<details>
<summary>Specification</summary>


The number of tiers is limited.


<pre><code><b>invariant</b> len(tiers) &lt;= <a href="DesignatedDealer.md#0x1_DesignatedDealer_MAX_NUM_TIERS">MAX_NUM_TIERS</a>;
</code></pre>


Tiers are ordered.


<pre><code><b>invariant</b> <b>forall</b> i in 0..len(tiers), j in 0..len(tiers) <b>where</b> i &lt; j: tiers[i] &lt; tiers[j];
</code></pre>



</details>

<a name="0x1_DesignatedDealer_ReceivedMintEvent"></a>

## Struct `ReceivedMintEvent`

Message for mint events


<pre><code><b>struct</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>
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
 The amount minted (in base units of <code>currency_code</code>)
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DesignatedDealer_ONE_DAY"></a>

Number of microseconds in a day


<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_ONE_DAY">ONE_DAY</a>: u64 = 86400000000;
</code></pre>



<a name="0x1_DesignatedDealer_EDEALER"></a>

The <code><a href="DesignatedDealer.md#0x1_DesignatedDealer">DesignatedDealer</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">EDEALER</a>: u64 = 0;
</code></pre>



<a name="0x1_DesignatedDealer_EINVALID_AMOUNT_FOR_TIER"></a>

The maximum amount of money that can be minted for the tier has been reached


<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_AMOUNT_FOR_TIER">EINVALID_AMOUNT_FOR_TIER</a>: u64 = 5;
</code></pre>



<a name="0x1_DesignatedDealer_EINVALID_MINT_AMOUNT"></a>

A zero mint amount was provided


<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_MINT_AMOUNT">EINVALID_MINT_AMOUNT</a>: u64 = 4;
</code></pre>



<a name="0x1_DesignatedDealer_EINVALID_TIER_ADDITION"></a>

The maximum number of tiers (4) has already been reached


<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_ADDITION">EINVALID_TIER_ADDITION</a>: u64 = 1;
</code></pre>



<a name="0x1_DesignatedDealer_EINVALID_TIER_INDEX"></a>

The tier index is out-of-bounds


<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_INDEX">EINVALID_TIER_INDEX</a>: u64 = 3;
</code></pre>



<a name="0x1_DesignatedDealer_EINVALID_TIER_START"></a>

The starting value for the tier overlaps with the tier below or above it


<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_START">EINVALID_TIER_START</a>: u64 = 2;
</code></pre>



<a name="0x1_DesignatedDealer_MAX_NUM_TIERS"></a>

The maximum number of tiers allowed


<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_MAX_NUM_TIERS">MAX_NUM_TIERS</a>: u64 = 4;
</code></pre>



<a name="0x1_DesignatedDealer_TIER_0_DEFAULT"></a>

Default FIAT amounts for tiers when a DD is created
These get scaled by coin specific scaling factor on tier setting


<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TIER_0_DEFAULT">TIER_0_DEFAULT</a>: u64 = 500000;
</code></pre>



<a name="0x1_DesignatedDealer_TIER_1_DEFAULT"></a>



<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TIER_1_DEFAULT">TIER_1_DEFAULT</a>: u64 = 5000000;
</code></pre>



<a name="0x1_DesignatedDealer_TIER_2_DEFAULT"></a>



<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TIER_2_DEFAULT">TIER_2_DEFAULT</a>: u64 = 50000000;
</code></pre>



<a name="0x1_DesignatedDealer_TIER_3_DEFAULT"></a>



<pre><code><b>const</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TIER_3_DEFAULT">TIER_3_DEFAULT</a>: u64 = 500000000;
</code></pre>



<a name="0x1_DesignatedDealer_publish_designated_dealer_credential"></a>

## Function `publish_designated_dealer_credential`

Publishes a <code><a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a></code> resource under <code>dd</code> with a <code><a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a></code>, <code>Preburn</code>, and default tiers for <code>CoinType</code>.
If <code>add_all_currencies = <b>true</b></code> this will add a <code><a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a></code>, <code>Preburn</code>,
and default tiers for each known currency at launch.


<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_publish_designated_dealer_credential">publish_designated_dealer_credential</a>&lt;CoinType&gt;(dd: &signer, tc_account: &signer, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_publish_designated_dealer_credential">publish_designated_dealer_credential</a>&lt;CoinType&gt;(
    dd: &signer,
    tc_account: &signer,
    add_all_currencies: bool,
) <b>acquires</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <a href="Roles.md#0x1_Roles_assert_designated_dealer">Roles::assert_designated_dealer</a>(dd);
    <b>assert</b>(!<b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(dd)), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">EDEALER</a>));
    move_to(dd, <a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a> { mint_event_handle: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>&gt;(dd) });
    <b>if</b> (add_all_currencies) {
        <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(dd, tc_account);
    } <b>else</b> {
        <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;CoinType&gt;(dd, tc_account);
    };
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<a name="0x1_DesignatedDealer_dd_addr$10"></a>
<b>let</b> dd_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(dd);
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDesignatedDealer">Roles::AbortsIfNotDesignatedDealer</a>{account: dd};
<b>aborts_if</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>include</b> <b>if</b> (add_all_currencies) <a href="DesignatedDealer.md#0x1_DesignatedDealer_AddCurrencyAbortsIf">AddCurrencyAbortsIf</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;{dd_addr: dd_addr}
        <b>else</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AddCurrencyAbortsIf">AddCurrencyAbortsIf</a>&lt;CoinType&gt;{dd_addr: dd_addr};
<b>modifies</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr);
<b>ensures</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr);
<b>modifies</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr), <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(dd_addr);
<b>ensures</b> <b>if</b> (add_all_currencies) <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(dd_addr) <b>else</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
</code></pre>



</details>

<a name="0x1_DesignatedDealer_add_currency"></a>

## Function `add_currency`

Adds the needed resources to the DD account <code>dd</code> in order to work with <code>CoinType</code>.
Public so that a currency can be added to a DD later on. Will require
multi-signer transactions in order to add a new currency to an existing DD.


<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;CoinType&gt;(dd: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_currency">add_currency</a>&lt;CoinType&gt;(dd: &signer, tc_account: &signer)
<b>acquires</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>let</b> dd_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(dd);
    <b>assert</b>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">EDEALER</a>));
    <a href="Diem.md#0x1_Diem_publish_preburn_to_account">Diem::publish_preburn_to_account</a>&lt;CoinType&gt;(dd, tc_account);
    <b>assert</b>(!<b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">EDEALER</a>));
    move_to(dd, <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt; {
        window_start: <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_microseconds">DiemTimestamp::now_microseconds</a>(),
        window_inflow: 0,
        tiers: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>(),
    });
    // Add tier amounts in base_units of CoinType
    <b>let</b> coin_scaling_factor = <a href="Diem.md#0x1_Diem_scaling_factor">Diem::scaling_factor</a>&lt;CoinType&gt;();
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account, dd_addr, <a href="DesignatedDealer.md#0x1_DesignatedDealer_TIER_0_DEFAULT">TIER_0_DEFAULT</a> * coin_scaling_factor);
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account, dd_addr, <a href="DesignatedDealer.md#0x1_DesignatedDealer_TIER_1_DEFAULT">TIER_1_DEFAULT</a> * coin_scaling_factor);
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account, dd_addr, <a href="DesignatedDealer.md#0x1_DesignatedDealer_TIER_2_DEFAULT">TIER_2_DEFAULT</a> * coin_scaling_factor);
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account, dd_addr, <a href="DesignatedDealer.md#0x1_DesignatedDealer_TIER_3_DEFAULT">TIER_3_DEFAULT</a> * coin_scaling_factor);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<a name="0x1_DesignatedDealer_dd_addr$11"></a>
<b>let</b> dd_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(dd);
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDesignatedDealer">Roles::AbortsIfNotDesignatedDealer</a>{account: dd};
<b>include</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AbortsIfNoDealer">AbortsIfNoDealer</a>{dd_addr: dd_addr};
<b>include</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AddCurrencyAbortsIf">AddCurrencyAbortsIf</a>&lt;CoinType&gt;{dd_addr: dd_addr};
<b>modifies</b> <b>global</b>&lt;<a href="Diem.md#0x1_Diem_Preburn">Diem::Preburn</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>modifies</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>ensures</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>ensures</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr) ==
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt; {
        window_start: <a href="DiemTimestamp.md#0x1_DiemTimestamp_spec_now_microseconds">DiemTimestamp::spec_now_microseconds</a>(),
        window_inflow: 0,
        tiers: <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers,
    };
<b>ensures</b> len(<b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers) == <a href="DesignatedDealer.md#0x1_DesignatedDealer_MAX_NUM_TIERS">MAX_NUM_TIERS</a>;
</code></pre>




<a name="0x1_DesignatedDealer_AddCurrencyAbortsIf"></a>


<pre><code><b>schema</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AddCurrencyAbortsIf">AddCurrencyAbortsIf</a>&lt;CoinType&gt; {
    dd_addr: address;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>include</b> <a href="Diem.md#0x1_Diem_AbortsIfNoCurrency">Diem::AbortsIfNoCurrency</a>&lt;CoinType&gt;;
    <b>aborts_if</b> <a href="Diem.md#0x1_Diem_is_synthetic_currency">Diem::is_synthetic_currency</a>&lt;CoinType&gt;() <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="Diem.md#0x1_Diem_Preburn">Diem::Preburn</a>&lt;CoinType&gt;&gt;(dd_addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotOperating">DiemTimestamp::AbortsIfNotOperating</a>;
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_add_tier"></a>

## Function `add_tier`



<pre><code><b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(tc_account: &signer, dd_addr: address, tier_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_add_tier">add_tier</a>&lt;CoinType&gt;(
    tc_account: &signer,
    dd_addr: address,
    tier_upperbound: u64
) <b>acquires</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>assert</b>(<b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">EDEALER</a>));
    <b>let</b> tiers = &<b>mut</b> borrow_global_mut&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers;
    <b>let</b> number_of_tiers = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(tiers);
    <b>assert</b>(number_of_tiers + 1 &lt;= <a href="DesignatedDealer.md#0x1_DesignatedDealer_MAX_NUM_TIERS">MAX_NUM_TIERS</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_ADDITION">EINVALID_TIER_ADDITION</a>));
    <b>if</b> (number_of_tiers &gt; 0) {
        <b>let</b> last_tier = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, number_of_tiers - 1);
        <b>assert</b>(last_tier &lt; tier_upperbound, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_START">EINVALID_TIER_START</a>));
    };
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(tiers, tier_upperbound);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AbortsIfNoTierInfo">AbortsIfNoTierInfo</a>&lt;CoinType&gt;;
<a name="0x1_DesignatedDealer_tier_info$12"></a>
<b>let</b> tier_info = <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<a name="0x1_DesignatedDealer_number_of_tiers$13"></a>
<b>let</b> number_of_tiers = len(tier_info.tiers);
<b>aborts_if</b> number_of_tiers == <a href="DesignatedDealer.md#0x1_DesignatedDealer_MAX_NUM_TIERS">MAX_NUM_TIERS</a> <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>aborts_if</b> number_of_tiers &gt; 0 && tier_info.tiers[number_of_tiers - 1] &gt;= tier_upperbound <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>modifies</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>ensures</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>ensures</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr) ==
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt; {
        window_start: <b>old</b>(<b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr)).window_start,
        window_inflow: <b>old</b>(<b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr)).window_inflow,
        tiers: concat_vector(<b>old</b>(<b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr)).tiers, singleton_vector(tier_upperbound)),
    };
</code></pre>




<a name="0x1_DesignatedDealer_AbortsIfNoTierInfo"></a>


<pre><code><b>schema</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AbortsIfNoTierInfo">AbortsIfNoTierInfo</a>&lt;CoinType&gt; {
    dd_addr: address;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_update_tier"></a>

## Function `update_tier`



<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_update_tier">update_tier</a>&lt;CoinType&gt;(tc_account: &signer, dd_addr: address, tier_index: u64, new_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_update_tier">update_tier</a>&lt;CoinType&gt;(
    tc_account: &signer,
    dd_addr: address,
    tier_index: u64,
    new_upperbound: u64
) <b>acquires</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>assert</b>(<b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">EDEALER</a>));
    <b>let</b> tiers = &<b>mut</b> borrow_global_mut&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers;
    <b>let</b> number_of_tiers = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(tiers);
    <b>assert</b>(tier_index &lt; number_of_tiers, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_INDEX">EINVALID_TIER_INDEX</a>));
    // Make sure that this new start for the tier is consistent <b>with</b> the tier above and below it.
    <b>let</b> tier = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, tier_index);
    <b>if</b> (*tier == new_upperbound) <b>return</b>;
    <b>if</b> (*tier &lt; new_upperbound) {
        <b>let</b> next_tier_index = tier_index + 1;
        <b>if</b> (next_tier_index &lt; number_of_tiers) {
            <b>assert</b>(
                new_upperbound &lt; *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, next_tier_index),
                <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_START">EINVALID_TIER_START</a>)
            );
        };
    };
    <b>if</b> (*tier &gt; new_upperbound && tier_index &gt; 0) {
        <b>let</b> prev_tier_index = tier_index - 1;
        <b>assert</b>(
            new_upperbound &gt; *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, prev_tier_index),
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_START">EINVALID_TIER_START</a>)
        );
    };
    *<a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(tiers, tier_index) = new_upperbound;
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AbortsIfNoTierInfo">AbortsIfNoTierInfo</a>&lt;CoinType&gt;;
<a name="0x1_DesignatedDealer_tier_info$14"></a>
<b>let</b> tier_info = <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>aborts_if</b> tier_index &gt;= len(tier_info.tiers) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>aborts_if</b> tier_index &gt; 0 && tier_info.tiers[tier_index - 1] &gt;= new_upperbound <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>aborts_if</b> tier_index + 1 &lt; len(tier_info.tiers) && tier_info.tiers[tier_index + 1] &lt;= new_upperbound <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>modifies</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>ensures</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>ensures</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr) ==
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt; {
        window_start: <b>old</b>(<b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr)).window_start,
        window_inflow: <b>old</b>(<b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr)).window_inflow,
        tiers: update_vector(<b>old</b>(<b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr)).tiers, tier_index, new_upperbound),
    };
</code></pre>



</details>

<a name="0x1_DesignatedDealer_tiered_mint"></a>

## Function `tiered_mint`



<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(tc_account: &signer, amount: u64, dd_addr: address, tier_index: u64): <a href="Diem.md#0x1_Diem_Diem">Diem::Diem</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(
    tc_account: &signer,
    amount: u64,
    dd_addr: address,
    tier_index: u64,
): <a href="Diem.md#0x1_Diem_Diem">Diem::Diem</a>&lt;CoinType&gt; <b>acquires</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>, <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>assert</b>(amount &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_MINT_AMOUNT">EINVALID_MINT_AMOUNT</a>));
    <b>assert</b>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">EDEALER</a>));
    <b>assert</b>(<b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">EDEALER</a>));

    <a href="DesignatedDealer.md#0x1_DesignatedDealer_validate_and_record_mint">validate_and_record_mint</a>&lt;CoinType&gt;(dd_addr, amount, tier_index);
    // Send <a href="DesignatedDealer.md#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr).mint_event_handle,
        <a href="DesignatedDealer.md#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a> {
            currency_code: <a href="Diem.md#0x1_Diem_currency_code">Diem::currency_code</a>&lt;CoinType&gt;(),
            destination_address: dd_addr,
            amount: amount,
        },
    );
    <a href="Diem.md#0x1_Diem_mint">Diem::mint</a>&lt;CoinType&gt;(tc_account, amount)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TieredMintAbortsIf">TieredMintAbortsIf</a>&lt;CoinType&gt;;
<b>modifies</b> <b>global</b>&lt;<a href="Diem.md#0x1_Diem_CurrencyInfo">Diem::CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="Diem.md#0x1_Diem_CurrencyInfo">Diem::CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
<b>modifies</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>ensures</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<b>ensures</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers == <b>old</b>(<b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr).tiers);
<a name="0x1_DesignatedDealer_dealer$15"></a>
<b>let</b> dealer = <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
<a name="0x1_DesignatedDealer_current_time$16"></a>
<b>let</b> current_time = <a href="DiemTimestamp.md#0x1_DiemTimestamp_spec_now_microseconds">DiemTimestamp::spec_now_microseconds</a>();
<a name="0x1_DesignatedDealer_currency_info$17"></a>
<b>let</b> currency_info = <b>global</b>&lt;<a href="Diem.md#0x1_Diem_CurrencyInfo">Diem::CurrencyInfo</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
<b>ensures</b> <b>old</b>(dealer.window_start) &lt;= dealer.window_start;
<b>ensures</b>
    dealer.window_start == current_time && dealer.window_inflow == amount ||
    (<b>old</b>(dealer.window_start) == dealer.window_start &&
        dealer.window_inflow == <b>old</b>(dealer.window_inflow) + amount);
<b>ensures</b> tier_index &lt; len(<b>old</b>(dealer).tiers);
<b>ensures</b> dealer.window_inflow &lt;= <b>old</b>(dealer).tiers[tier_index];
<b>ensures</b> result.value == amount;
<b>ensures</b> currency_info == update_field(<b>old</b>(currency_info), total_value, <b>old</b>(currency_info.total_value) + amount);
</code></pre>




<a name="0x1_DesignatedDealer_TieredMintAbortsIf"></a>


<pre><code><b>schema</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TieredMintAbortsIf">TieredMintAbortsIf</a>&lt;CoinType&gt; {
    tc_account: signer;
    dd_addr: address;
    amount: u64;
    tier_index: u64;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
    <b>aborts_if</b> amount == 0 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>include</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AbortsIfNoDealer">AbortsIfNoDealer</a>;
    <b>include</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AbortsIfNoTierInfo">AbortsIfNoTierInfo</a>&lt;CoinType&gt;;
    <a name="0x1_DesignatedDealer_tier_info$8"></a>
    <b>let</b> tier_info = <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
    <b>aborts_if</b> tier_index &gt;= len(tier_info.tiers) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <a name="0x1_DesignatedDealer_new_amount$9"></a>
    <b>let</b> new_amount = <b>if</b> (<a href="DiemTimestamp.md#0x1_DiemTimestamp_spec_now_microseconds">DiemTimestamp::spec_now_microseconds</a>() &lt;= tier_info.window_start + <a href="DesignatedDealer.md#0x1_DesignatedDealer_ONE_DAY">ONE_DAY</a>) { tier_info.window_inflow + amount } <b>else</b> { amount };
    <b>aborts_if</b> new_amount &gt; tier_info.tiers[tier_index] <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotOperating">DiemTimestamp::AbortsIfNotOperating</a>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Diem.md#0x1_Diem_MintCapability">Diem::MintCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(tc_account)) <b>with</b> <a href="Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>;
    <b>include</b> <a href="Diem.md#0x1_Diem_MintAbortsIf">Diem::MintAbortsIf</a>&lt;CoinType&gt;{value: amount};
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr: address): bool {
    <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr);
</code></pre>



</details>

<a name="0x1_DesignatedDealer_validate_and_record_mint"></a>

## Function `validate_and_record_mint`

Validate and record the minting of <code>amount</code> of <code>CoinType</code> coins against
the <code><a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;</code> for the DD at <code>dd_addr</code>. Aborts if an invalid
<code>tier_index</code> is supplied, or if the inflow over the time period exceeds
that amount that can be minted according to the bounds for the <code>tier_index</code> tier.


<pre><code><b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_validate_and_record_mint">validate_and_record_mint</a>&lt;CoinType&gt;(dd_addr: address, amount: u64, tier_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_validate_and_record_mint">validate_and_record_mint</a>&lt;CoinType&gt;(dd_addr: address, amount: u64, tier_index: u64)
<b>acquires</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a> {
    <b>let</b> tier_info = borrow_global_mut&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_reset_window">reset_window</a>(tier_info);
    <b>let</b> cur_inflow = tier_info.window_inflow;
    <b>let</b> tiers = &tier_info.tiers;
    <b>let</b> number_of_tiers = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(tiers);
    <b>assert</b>(tier_index &lt; number_of_tiers, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_INDEX">EINVALID_TIER_INDEX</a>));
    <b>let</b> tier_upperbound: u64 = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, tier_index);
    <b>assert</b>(amount &lt;= tier_upperbound, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_AMOUNT_FOR_TIER">EINVALID_AMOUNT_FOR_TIER</a>));
    <b>assert</b>(cur_inflow &lt;= tier_upperbound - amount, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_AMOUNT_FOR_TIER">EINVALID_AMOUNT_FOR_TIER</a>));
    tier_info.window_inflow = cur_inflow + amount;
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>modifies</b> <b>global</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;&gt;(dd_addr);
</code></pre>



</details>

<a name="0x1_DesignatedDealer_reset_window"></a>

## Function `reset_window`



<pre><code><b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_reset_window">reset_window</a>&lt;CoinType&gt;(tier_info: &<b>mut</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">DesignatedDealer::TierInfo</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_reset_window">reset_window</a>&lt;CoinType&gt;(tier_info: &<b>mut</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;CoinType&gt;) {
    <b>let</b> current_time = <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_microseconds">DiemTimestamp::now_microseconds</a>();
    <b>if</b> (current_time &gt; <a href="DesignatedDealer.md#0x1_DesignatedDealer_ONE_DAY">ONE_DAY</a> && current_time - <a href="DesignatedDealer.md#0x1_DesignatedDealer_ONE_DAY">ONE_DAY</a> &gt; tier_info.window_start) {
        tier_info.window_start = current_time;
        tier_info.window_inflow = 0;
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotOperating">DiemTimestamp::AbortsIfNotOperating</a>;
<a name="0x1_DesignatedDealer_current_time$18"></a>
<b>let</b> current_time = <a href="DiemTimestamp.md#0x1_DiemTimestamp_spec_now_microseconds">DiemTimestamp::spec_now_microseconds</a>();
<b>ensures</b>
    <b>if</b> (current_time &gt; <a href="DesignatedDealer.md#0x1_DesignatedDealer_ONE_DAY">ONE_DAY</a> && current_time - <a href="DesignatedDealer.md#0x1_DesignatedDealer_ONE_DAY">ONE_DAY</a> &gt; <b>old</b>(tier_info).window_start)
        tier_info == update_field(update_field(<b>old</b>(tier_info),
            window_start, current_time),
            window_inflow, 0)
    <b>else</b>
        tier_info == <b>old</b>(tier_info);
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



resource struct Dealer persists after publication


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>] <b>forall</b> addr: address <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(addr)): <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(addr);
</code></pre>


TierInfo persists


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>] <b>forall</b> addr: address, coin_type: type <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;coin_type&gt;&gt;(addr)):
    <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">TierInfo</a>&lt;coin_type&gt;&gt;(addr);
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/master/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/master/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/master/dips/dip-2.md#permissions
