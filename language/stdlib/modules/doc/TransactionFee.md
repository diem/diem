
<a name="0x1_TransactionFee"></a>

# Module `0x1::TransactionFee`



-  [Resource <code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code>](#0x1_TransactionFee_TransactionFee)
-  [Const <code><a href="TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">ETRANSACTION_FEE</a></code>](#0x1_TransactionFee_ETRANSACTION_FEE)
-  [Function <code>initialize</code>](#0x1_TransactionFee_initialize)
-  [Function <code>is_coin_initialized</code>](#0x1_TransactionFee_is_coin_initialized)
-  [Function <code>is_initialized</code>](#0x1_TransactionFee_is_initialized)
-  [Function <code>add_txn_fee_currency</code>](#0x1_TransactionFee_add_txn_fee_currency)
-  [Function <code>pay_fee</code>](#0x1_TransactionFee_pay_fee)
-  [Function <code>burn_fees</code>](#0x1_TransactionFee_burn_fees)
-  [Module Specification](#@Module_Specification_0)


<a name="0x1_TransactionFee_TransactionFee"></a>

## Resource `TransactionFee`

The <code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code> resource holds a preburn resource for each
fiat <code>CoinType</code> that can be collected as a transaction fee.


<pre><code><b>resource</b> <b>struct</b> <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;
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

<a name="0x1_TransactionFee_ETRANSACTION_FEE"></a>

## Const `ETRANSACTION_FEE`

A <code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code> resource is not in the required state


<pre><code><b>const</b> <a href="TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">ETRANSACTION_FEE</a>: u64 = 0;
</code></pre>



<a name="0x1_TransactionFee_initialize"></a>

## Function `initialize`

Called in genesis. Sets up the needed resources to collect transaction fees from the
<code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code> resource with the TreasuryCompliance account.


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_initialize">initialize</a>(lr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    // accept fees in all the currencies
    <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(lr_account, tc_account);
    <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(lr_account, tc_account);
    <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(lr_account, tc_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="TransactionFee.md#0x1_TransactionFee_AddTxnFeeCurrencyAbortsIf">AddTxnFeeCurrencyAbortsIf</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;;
<b>include</b> <a href="TransactionFee.md#0x1_TransactionFee_AddTxnFeeCurrencyAbortsIf">AddTxnFeeCurrencyAbortsIf</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;;
<b>include</b> <a href="TransactionFee.md#0x1_TransactionFee_AddTxnFeeCurrencyAbortsIf">AddTxnFeeCurrencyAbortsIf</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;;
<b>ensures</b> <a href="TransactionFee.md#0x1_TransactionFee_is_initialized">is_initialized</a>();
<b>ensures</b> <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;().balance.value == 0;
<b>ensures</b> <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;().balance.value == 0;
<b>ensures</b> <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;().balance.value == 0;
</code></pre>




<a name="0x1_TransactionFee_AddTxnFeeCurrencyAbortsIf"></a>


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_AddTxnFeeCurrencyAbortsIf">AddTxnFeeCurrencyAbortsIf</a>&lt;CoinType&gt; {
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;CoinType&gt;;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
}
</code></pre>



</details>

<a name="0x1_TransactionFee_is_coin_initialized"></a>

## Function `is_coin_initialized`



<pre><code><b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;(): bool {
    <b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_TransactionFee_is_initialized"></a>

## Function `is_initialized`



<pre><code><b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_is_initialized">is_initialized</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_is_initialized">is_initialized</a>(): bool {
    <a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;() && <a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;() && <a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;()
}
</code></pre>



</details>

<a name="0x1_TransactionFee_add_txn_fee_currency"></a>

## Function `add_txn_fee_currency`

Sets ups the needed transaction fee state for a given <code>CoinType</code> currency by
(1) configuring <code>lr_account</code> to accept <code>CoinType</code>
(2) publishing a wrapper of the <code>Preburn&lt;CoinType&gt;</code> resource under <code>lr_account</code>


<pre><code><b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(lr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(
    lr_account: &signer,
    tc_account: &signer,
) {
    <a href="Libra.md#0x1_Libra_assert_is_currency">Libra::assert_is_currency</a>&lt;CoinType&gt;();
    <b>assert</b>(
        !<b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">ETRANSACTION_FEE</a>)
    );
    move_to(
        lr_account,
        <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt; {
            balance: <a href="Libra.md#0x1_Libra_zero">Libra::zero</a>(),
            preburn: <a href="Libra.md#0x1_Libra_create_preburn">Libra::create_preburn</a>(tc_account)
        }
    )
}
</code></pre>



</details>

<a name="0x1_TransactionFee_pay_fee"></a>

## Function `pay_fee`

Deposit <code>coin</code> into the transaction fees bucket


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">pay_fee</a>&lt;CoinType&gt;(coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">pay_fee</a>&lt;CoinType&gt;(coin: <a href="Libra.md#0x1_Libra">Libra</a>&lt;CoinType&gt;) <b>acquires</b> <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <b>assert</b>(<a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;(), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">ETRANSACTION_FEE</a>));
    <b>let</b> fees = borrow_global_mut&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(
        <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()
    );
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> fees.balance, coin)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>aborts_if</b> !<a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;() <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
<a name="0x1_TransactionFee_fees$13"></a>
<b>let</b> fees = <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;CoinType&gt;().balance;
<b>include</b> <a href="Libra.md#0x1_Libra_DepositAbortsIf">Libra::DepositAbortsIf</a>&lt;CoinType&gt;{coin: fees, check: coin};
<b>ensures</b> fees.value == <b>old</b>(fees.value) + coin.value;
</code></pre>



</details>

<a name="0x1_TransactionFee_burn_fees"></a>

## Function `burn_fees`

Preburns the transaction fees collected in the <code>CoinType</code> currency.
If the <code>CoinType</code> is LBR, it unpacks the coin and preburns the
underlying fiat.


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_burn_fees">burn_fees</a>&lt;CoinType&gt;(tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_burn_fees">burn_fees</a>&lt;CoinType&gt;(
    tc_account: &signer,
) <b>acquires</b> <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <b>assert</b>(<a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;(), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">ETRANSACTION_FEE</a>));
    <b>let</b> fee_address =  <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>let</b> tc_address = <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>();
    <b>if</b> (<a href="LBR.md#0x1_LBR_is_lbr">LBR::is_lbr</a>&lt;CoinType&gt;()) {
        // extract fees
        <b>let</b> fees = borrow_global_mut&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(fee_address);
        <b>let</b> coins = <a href="Libra.md#0x1_Libra_withdraw_all">Libra::withdraw_all</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(&<b>mut</b> fees.balance);
        <b>let</b> (coin1, coin2) = <a href="LBR.md#0x1_LBR_unpack">LBR::unpack</a>(coins);
        // burn
        <b>let</b> coin1_burn_cap = <a href="Libra.md#0x1_Libra_remove_burn_capability">Libra::remove_burn_capability</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(tc_account);
        <b>let</b> coin2_burn_cap = <a href="Libra.md#0x1_Libra_remove_burn_capability">Libra::remove_burn_capability</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(tc_account);
        <a href="Libra.md#0x1_Libra_burn_now">Libra::burn_now</a>(
            coin1,
            &<b>mut</b> borrow_global_mut&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(fee_address).preburn,
            tc_address,
            &coin1_burn_cap
        );
        <a href="Libra.md#0x1_Libra_burn_now">Libra::burn_now</a>(
            coin2,
            &<b>mut</b> borrow_global_mut&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(fee_address).preburn,
            tc_address,
            &coin2_burn_cap
        );
        <a href="Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>(tc_account, coin1_burn_cap);
        <a href="Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>(tc_account, coin2_burn_cap);
    } <b>else</b> {
        // extract fees
        <b>let</b> fees = borrow_global_mut&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(fee_address);
        <b>let</b> coin = <a href="Libra.md#0x1_Libra_withdraw_all">Libra::withdraw_all</a>(&<b>mut</b> fees.balance);
        <b>let</b> burn_cap = <a href="Libra.md#0x1_Libra_remove_burn_capability">Libra::remove_burn_capability</a>&lt;CoinType&gt;(tc_account);
        // burn
        <a href="Libra.md#0x1_Libra_burn_now">Libra::burn_now</a>(
            coin,
            &<b>mut</b> fees.preburn,
            tc_address,
            &burn_cap
        );
        <a href="Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>(tc_account, burn_cap);
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


> TODO: this times out and likely is also not fully correct yet.


<pre><code>pragma verify = <b>false</b>;
</code></pre>


Must abort if the account does not have the TreasuryCompliance role [B12].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>aborts_if</b> !<a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;() <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
<b>include</b> <b>if</b> (<a href="LBR.md#0x1_LBR_spec_is_lbr">LBR::spec_is_lbr</a>&lt;CoinType&gt;()) <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesLBR">BurnFeesLBR</a> <b>else</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesNotLBR">BurnFeesNotLBR</a>&lt;CoinType&gt;;
</code></pre>


The correct amount of fees is burnt and subtracted from market cap.


<pre><code><b>ensures</b> <a href="Libra.md#0x1_Libra_spec_market_cap">Libra::spec_market_cap</a>&lt;CoinType&gt;()
    == <b>old</b>(<a href="Libra.md#0x1_Libra_spec_market_cap">Libra::spec_market_cap</a>&lt;CoinType&gt;()) - <b>old</b>(<a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;CoinType&gt;().balance.value);
</code></pre>


All the fees is burnt so the balance becomes 0.


<pre><code><b>ensures</b> <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;CoinType&gt;().balance.value == 0;
</code></pre>


Specification of the case where burn type is LBR.


<a name="0x1_TransactionFee_BurnFeesLBR"></a>


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesLBR">BurnFeesLBR</a> {
    tc_account: signer;
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoBurnCapability">Libra::AbortsIfNoBurnCapability</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;{account: tc_account};
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoBurnCapability">Libra::AbortsIfNoBurnCapability</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;{account: tc_account};
    <a name="0x1_TransactionFee_lbr_fees$7"></a>
    <b>let</b> lbr_fees = <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;();
    <b>include</b> <a href="LBR.md#0x1_LBR_UnpackAbortsIf">LBR::UnpackAbortsIf</a>{coin: lbr_fees.balance};
    <a name="0x1_TransactionFee_coin1_fees$8"></a>
    <b>let</b> coin1_fees = <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;();
    <a name="0x1_TransactionFee_coin1$9"></a>
    <b>let</b> coin1 = <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;{value: <a href="LBR.md#0x1_LBR_spec_unpack_coin1">LBR::spec_unpack_coin1</a>(lbr_fees.balance)};
    <b>include</b> <a href="Libra.md#0x1_Libra_BurnNowAbortsIf">Libra::BurnNowAbortsIf</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;{coin: coin1, preburn: coin1_fees.preburn};
    <a name="0x1_TransactionFee_coin2_fees$10"></a>
    <b>let</b> coin2_fees = <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;();
    <a name="0x1_TransactionFee_coin2$11"></a>
    <b>let</b> coin2 = <a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;{value: <a href="LBR.md#0x1_LBR_spec_unpack_coin2">LBR::spec_unpack_coin2</a>(lbr_fees.balance)};
    <b>include</b> <a href="Libra.md#0x1_Libra_BurnNowAbortsIf">Libra::BurnNowAbortsIf</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;{coin: coin2, preburn: coin2_fees.preburn};
}
</code></pre>


Specification of the case where burn type is not LBR.


<a name="0x1_TransactionFee_BurnFeesNotLBR"></a>


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesNotLBR">BurnFeesNotLBR</a>&lt;CoinType&gt; {
    tc_account: signer;
}
</code></pre>


Must abort if the account does not have BurnCapability [B12].


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesNotLBR">BurnFeesNotLBR</a>&lt;CoinType&gt; {
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoBurnCapability">Libra::AbortsIfNoBurnCapability</a>&lt;CoinType&gt;{account: tc_account};
    <a name="0x1_TransactionFee_fees$12"></a>
    <b>let</b> fees = <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;CoinType&gt;();
    <b>include</b> <a href="Libra.md#0x1_Libra_BurnNowAbortsIf">Libra::BurnNowAbortsIf</a>&lt;CoinType&gt;{coin: fees.balance, preburn: fees.preburn};
}
</code></pre>


tc_account retrieves BurnCapability [B12]. BurnCapability is not transferrable [D12].


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesNotLBR">BurnFeesNotLBR</a>&lt;CoinType&gt; {
    <b>ensures</b> <b>exists</b>&lt;<a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(tc_account));
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="TransactionFee.md#0x1_TransactionFee_is_initialized">is_initialized</a>();
</code></pre>




<a name="0x1_TransactionFee_transaction_fee"></a>


<pre><code><b>define</b> <a href="TransactionFee.md#0x1_TransactionFee_transaction_fee">transaction_fee</a>&lt;CoinType&gt;(): <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt; {
borrow_global&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>
