
<a name="0x1_TransactionFee"></a>

# Module `0x1::TransactionFee`

### Table of Contents

-  [Resource `TransactionFee`](#0x1_TransactionFee_TransactionFee)
-  [Function `initialize`](#0x1_TransactionFee_initialize)
-  [Function `add_txn_fee_currency`](#0x1_TransactionFee_add_txn_fee_currency)
-  [Function `pay_fee`](#0x1_TransactionFee_pay_fee)
-  [Function `burn_fees`](#0x1_TransactionFee_burn_fees)
-  [Function `preburn_burn_fees`](#0x1_TransactionFee_preburn_burn_fees)



<a name="0x1_TransactionFee_TransactionFee"></a>

## Resource `TransactionFee`

The
<code><a href="#0x1_TransactionFee">TransactionFee</a></code> resource holds a preburn resource for each
fiat
<code>CoinType</code> that can be collected as a transaction fee.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>balance: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>preburn: <a href="Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TransactionFee_initialize"></a>

## Function `initialize`

Called in genesis. Sets up the needed resources to collect transaction fees from the
<code><a href="#0x1_TransactionFee">TransactionFee</a></code> resource with the TreasuryCompliance account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_initialize">initialize</a>(assoc_account: &signer, tc_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_initialize">initialize</a>(
    assoc_account: &signer,
    tc_capability: &Capability&lt;TreasuryComplianceRole&gt;,
) {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(assoc_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(),
        0
    );

    // accept fees in all the currencies
    <a href="#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(assoc_account, tc_capability);
    <a href="#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(assoc_account, tc_capability);
    <a href="#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(assoc_account, tc_capability);
}
</code></pre>



</details>

<a name="0x1_TransactionFee_add_txn_fee_currency"></a>

## Function `add_txn_fee_currency`

Sets ups the needed transaction fee state for a given
<code>CoinType</code> currency by
(1) configuring
<code>fee_account</code> to accept
<code>CoinType</code>
(2) publishing a wrapper of the
<code>Preburn&lt;CoinType&gt;</code> resource under
<code>fee_account</code>


<pre><code><b>fun</b> <a href="#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(assoc_account: &signer, tc_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(
    assoc_account: &signer,
    tc_capability: &Capability&lt;TreasuryComplianceRole&gt;,
) {
    move_to(
        assoc_account,
        <a href="#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt; {
            balance: <a href="Libra.md#0x1_Libra_zero">Libra::zero</a>(),
            preburn: <a href="Libra.md#0x1_Libra_create_preburn">Libra::create_preburn</a>(tc_capability)
        }
    )
}
</code></pre>



</details>

<a name="0x1_TransactionFee_pay_fee"></a>

## Function `pay_fee`

Deposit
<code>coin</code> into the transaction fees bucket


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_pay_fee">pay_fee</a>&lt;CoinType&gt;(coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_pay_fee">pay_fee</a>&lt;CoinType&gt;(coin: <a href="Libra.md#0x1_Libra">Libra</a>&lt;CoinType&gt;) <b>acquires</b> <a href="#0x1_TransactionFee">TransactionFee</a> {
    <b>let</b> fees = borrow_global_mut&lt;<a href="#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(
        <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()
    );
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> fees.balance, coin)
}
</code></pre>



</details>

<a name="0x1_TransactionFee_burn_fees"></a>

## Function `burn_fees`

Preburns the transaction fees collected in the
<code>CoinType</code> currency.
If the
<code>CoinType</code> is LBR, it unpacks the coin and preburns the
underlying fiat.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_burn_fees">burn_fees</a>&lt;CoinType&gt;(tc_account: &signer, tc_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_burn_fees">burn_fees</a>&lt;CoinType&gt;(
    tc_account: &signer,
    tc_capability: &Capability&lt;TreasuryComplianceRole&gt;,
) <b>acquires</b> <a href="#0x1_TransactionFee">TransactionFee</a> {
    <b>let</b> fee_address =  <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>if</b> (<a href="LBR.md#0x1_LBR_is_lbr">LBR::is_lbr</a>&lt;CoinType&gt;()) {
        // extract fees
        <b>let</b> fees = borrow_global_mut&lt;<a href="#0x1_TransactionFee">TransactionFee</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(fee_address);
        <b>let</b> coins = <a href="Libra.md#0x1_Libra_withdraw_all">Libra::withdraw_all</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(&<b>mut</b> fees.balance);
        <b>let</b> (coin1, coin2) = <a href="LBR.md#0x1_LBR_unpack">LBR::unpack</a>(tc_account, coins);
        // burn
        <b>let</b> coin1_burn_cap = <a href="Libra.md#0x1_Libra_remove_burn_capability">Libra::remove_burn_capability</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(tc_account);
        <b>let</b> coin2_burn_cap = <a href="Libra.md#0x1_Libra_remove_burn_capability">Libra::remove_burn_capability</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(tc_account);
        <a href="#0x1_TransactionFee_preburn_burn_fees">preburn_burn_fees</a>(
            &coin1_burn_cap,
            borrow_global_mut&lt;<a href="#0x1_TransactionFee">TransactionFee</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(fee_address),
            coin1
        );
        <a href="#0x1_TransactionFee_preburn_burn_fees">preburn_burn_fees</a>(
            &coin2_burn_cap,
            borrow_global_mut&lt;<a href="#0x1_TransactionFee">TransactionFee</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(fee_address),
            coin2
        );
        <a href="Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>(tc_account, coin1_burn_cap, tc_capability);
        <a href="Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>(tc_account, coin2_burn_cap, tc_capability);
    } <b>else</b> {
        // extract fees
        <b>let</b> fees = borrow_global_mut&lt;<a href="#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(fee_address);
        <b>let</b> coin = <a href="Libra.md#0x1_Libra_withdraw_all">Libra::withdraw_all</a>(&<b>mut</b> fees.balance);
        // burn
        <b>let</b> burn_cap = <a href="Libra.md#0x1_Libra_remove_burn_capability">Libra::remove_burn_capability</a>&lt;CoinType&gt;(tc_account);
        <a href="#0x1_TransactionFee_preburn_burn_fees">preburn_burn_fees</a>(&burn_cap, fees, coin);
        <a href="Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>(tc_account, burn_cap, tc_capability);
    }
}
</code></pre>



</details>

<a name="0x1_TransactionFee_preburn_burn_fees"></a>

## Function `preburn_burn_fees`

Preburn
<code>coin</code> to the
<code>Preburn</code> inside
<code>fees</code>, then immediately burn them using
<code>burn_cap</code>.


<pre><code><b>fun</b> <a href="#0x1_TransactionFee_preburn_burn_fees">preburn_burn_fees</a>&lt;CoinType&gt;(burn_cap: &<a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;, fees: &<b>mut</b> <a href="#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;CoinType&gt;, coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_TransactionFee_preburn_burn_fees">preburn_burn_fees</a>&lt;CoinType&gt;(
    burn_cap: &BurnCapability&lt;CoinType&gt;,
    fees: &<b>mut</b> <a href="#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;,
    coin: <a href="Libra.md#0x1_Libra">Libra</a>&lt;CoinType&gt;
) {
    <b>let</b> tc_address = <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>();
    <b>let</b> preburn = &<b>mut</b> fees.preburn;
    <a href="Libra.md#0x1_Libra_preburn_with_resource">Libra::preburn_with_resource</a>(coin, preburn, tc_address);
    <a href="Libra.md#0x1_Libra_burn_with_resource_cap">Libra::burn_with_resource_cap</a>(preburn, tc_address, burn_cap)
}
</code></pre>



</details>
