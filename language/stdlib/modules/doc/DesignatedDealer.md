
<a name="0x1_DesignatedDealer"></a>

# Module `0x1::DesignatedDealer`

### Table of Contents

-  [Struct `Dealer`](#0x1_DesignatedDealer_Dealer)
-  [Struct `ReceivedMintEvent`](#0x1_DesignatedDealer_ReceivedMintEvent)
-  [Function `publish_designated_dealer_credential`](#0x1_DesignatedDealer_publish_designated_dealer_credential)
-  [Function `add_tier_`](#0x1_DesignatedDealer_add_tier_)
-  [Function `add_tier`](#0x1_DesignatedDealer_add_tier)
-  [Function `update_tier_`](#0x1_DesignatedDealer_update_tier_)
-  [Function `update_tier`](#0x1_DesignatedDealer_update_tier)
-  [Function `tiered_mint_`](#0x1_DesignatedDealer_tiered_mint_)
-  [Function `tiered_mint`](#0x1_DesignatedDealer_tiered_mint)
-  [Function `exists_at`](#0x1_DesignatedDealer_exists_at)
-  [Function `reset_window`](#0x1_DesignatedDealer_reset_window)
-  [Function `window_length`](#0x1_DesignatedDealer_window_length)



<a name="0x1_DesignatedDealer_Dealer"></a>

## Struct `Dealer`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a>
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
<dt>

<code>mint_event_handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_DesignatedDealer_ReceivedMintEvent">DesignatedDealer::ReceivedMintEvent</a>&gt;</code>
</dt>
<dd>
 Handle for mint events
</dd>
</dl>


</details>

<a name="0x1_DesignatedDealer_ReceivedMintEvent"></a>

## Struct `ReceivedMintEvent`



<pre><code><b>struct</b> <a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>destination_address: address</code>
</dt>
<dd>

</dd>
<dt>

<code>amount: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DesignatedDealer_publish_designated_dealer_credential"></a>

## Function `publish_designated_dealer_credential`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_publish_designated_dealer_credential">publish_designated_dealer_credential</a>(association: &signer, dd: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_publish_designated_dealer_credential">publish_designated_dealer_credential</a>(association: &signer, dd: &signer) {
    // TODO: this should check for AssocRoot in the future
    <a href="Association.md#0x1_Association_assert_is_association">Association::assert_is_association</a>(association);
    move_to(
        dd,
        <a href="#0x1_DesignatedDealer_Dealer">Dealer</a> {
            window_start: <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>(),
            window_inflow: 0,
            tiers: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>(),
            mint_event_handle: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>&gt;(dd),
        }
    )
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_add_tier_"></a>

## Function `add_tier_`



<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_add_tier_">add_tier_</a>(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>, next_tier_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_add_tier_">add_tier_</a>(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a>, next_tier_upperbound: u64) {
    <b>let</b> tiers = &<b>mut</b> dealer.tiers;
    <b>let</b> number_of_tiers: u64 = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(tiers);
    // INVALID_TIER_ADDITION
    <b>assert</b>(number_of_tiers &lt;= 4, 31);
    <b>if</b> (number_of_tiers &gt; 1) {
        <b>let</b> prev_tier = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, number_of_tiers - 1);
        // INVALID_TIER_START
        <b>assert</b>(prev_tier &lt; next_tier_upperbound, 4);
    };
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(tiers, next_tier_upperbound);
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_add_tier"></a>

## Function `add_tier`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_add_tier">add_tier</a>(blessed: &signer, addr: address, tier_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_add_tier">add_tier</a>(blessed: &signer, addr: address, tier_upperbound: u64
) <b>acquires</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a> {
    <a href="Association.md#0x1_Association_assert_account_is_blessed">Association::assert_account_is_blessed</a>(blessed);
    <b>let</b> dealer = borrow_global_mut&lt;<a href="#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(addr);
    <a href="#0x1_DesignatedDealer_add_tier_">add_tier_</a>(dealer, tier_upperbound)
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_update_tier_"></a>

## Function `update_tier_`



<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_update_tier_">update_tier_</a>(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>, tier_index: u64, new_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_update_tier_">update_tier_</a>(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a>, tier_index: u64, new_upperbound: u64) {
    <b>let</b> tiers = &<b>mut</b> dealer.tiers;
    <b>let</b> number_of_tiers = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(tiers);
    // INVALID_TIER_INDEX
    <b>assert</b>(tier_index &lt;= 3, 3); // max 4 tiers allowed
    <b>assert</b>(tier_index &lt; number_of_tiers, 3);
    // Make sure that this new start for the tier is consistent
    // with the tier above it.
    <b>let</b> next_tier = tier_index + 1;
    <b>if</b> (next_tier &lt; number_of_tiers) {
        // INVALID_TIER_START
        <b>assert</b>(new_upperbound &lt; *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, next_tier), 4);
    };
    <b>let</b> tier_mut = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(tiers, tier_index);
    *tier_mut = new_upperbound;
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_update_tier"></a>

## Function `update_tier`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_update_tier">update_tier</a>(blessed: &signer, addr: address, tier_index: u64, new_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_update_tier">update_tier</a>(
    blessed: &signer, addr: address, tier_index: u64, new_upperbound: u64
) <b>acquires</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a> {
    <a href="Association.md#0x1_Association_assert_account_is_blessed">Association::assert_account_is_blessed</a>(blessed);
    <b>let</b> dealer = borrow_global_mut&lt;<a href="#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(addr);
    <a href="#0x1_DesignatedDealer_update_tier_">update_tier_</a>(dealer, tier_index, new_upperbound)
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_tiered_mint_"></a>

## Function `tiered_mint_`



<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_tiered_mint_">tiered_mint_</a>(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>, amount: u64, tier_index: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_tiered_mint_">tiered_mint_</a>(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a>, amount: u64, tier_index: u64): bool {
    // INVALID TIER_INDEX (<b>if</b> tier is 4, can mint unlimited)
    <b>assert</b>(tier_index &lt;= 4, 66);
    <a href="#0x1_DesignatedDealer_reset_window">reset_window</a>(dealer);
    <b>let</b> cur_inflow = *&dealer.window_inflow;
    <b>let</b> tiers = &<b>mut</b> dealer.tiers;
    // If the tier_index is one past the bounded tiers, minting is unbounded
    <b>let</b> number_of_tiers = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(tiers);
    <b>let</b> tier_check = &<b>mut</b> <b>false</b>;
    <b>if</b> (tier_index == number_of_tiers) {
        *tier_check = <b>true</b>;
    } <b>else</b> {
        <b>let</b> tier_upperbound: u64 = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(tiers, tier_index);
        *tier_check = (cur_inflow + amount &lt;= tier_upperbound);
    };
    <b>if</b> (*tier_check) {
        dealer.window_inflow = cur_inflow + amount;
    };
    *tier_check
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_tiered_mint"></a>

## Function `tiered_mint`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(blessed: &signer, amount: u64, dd_addr: address, tier_index: u64): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_tiered_mint">tiered_mint</a>&lt;CoinType&gt;(
    blessed: &signer, amount: u64, dd_addr: address, tier_index: u64
): <a href="Libra.md#0x1_Libra">Libra</a>&lt;CoinType&gt; <b>acquires</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a> {
    <a href="Association.md#0x1_Association_assert_account_is_blessed">Association::assert_account_is_blessed</a>(blessed);
    // NOT_A_DD
    <b>assert</b>(<a href="#0x1_DesignatedDealer_exists_at">exists_at</a>(dd_addr), 1);
    <b>let</b> tier_check = <a href="#0x1_DesignatedDealer_tiered_mint_">tiered_mint_</a>(borrow_global_mut&lt;<a href="#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr), amount, tier_index);
    // INVALID_AMOUNT_FOR_TIER
    <b>assert</b>(tier_check, 5);
    // Send <a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(dd_addr).mint_event_handle,
        <a href="#0x1_DesignatedDealer_ReceivedMintEvent">ReceivedMintEvent</a> {
            destination_address: dd_addr,
            amount: amount,
        },
    );
    <a href="Libra.md#0x1_Libra_mint">Libra::mint</a>&lt;CoinType&gt;(blessed, amount)
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_exists_at">exists_at</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DesignatedDealer_exists_at">exists_at</a>(addr: address): bool {
    exists&lt;<a href="#0x1_DesignatedDealer_Dealer">Dealer</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_reset_window"></a>

## Function `reset_window`



<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_reset_window">reset_window</a>(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_reset_window">reset_window</a>(dealer: &<b>mut</b> <a href="#0x1_DesignatedDealer_Dealer">Dealer</a>) {
    <b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
    <b>if</b> (current_time &gt; dealer.window_start + <a href="#0x1_DesignatedDealer_window_length">window_length</a>()) {
        dealer.window_start = current_time;
        dealer.window_inflow = 0;
    }
}
</code></pre>



</details>

<a name="0x1_DesignatedDealer_window_length"></a>

## Function `window_length`



<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_window_length">window_length</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_DesignatedDealer_window_length">window_length</a>(): u64 {
    // number of microseconds in a day
    86400000000
}
</code></pre>



</details>
