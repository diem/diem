
<a name="0x1_TransactionFee"></a>

# Module `0x1::TransactionFee`

Functions to initialize, accumulated, and burn transaction fees.


-  [Resource `TransactionFee`](#0x1_TransactionFee_TransactionFee)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_TransactionFee_initialize)
-  [Function `is_coin_initialized`](#0x1_TransactionFee_is_coin_initialized)
-  [Function `is_initialized`](#0x1_TransactionFee_is_initialized)
-  [Function `add_txn_fee_currency`](#0x1_TransactionFee_add_txn_fee_currency)
-  [Function `pay_fee`](#0x1_TransactionFee_pay_fee)
-  [Function `burn_fees`](#0x1_TransactionFee_burn_fees)
    -  [Specification of the case where burn type is LBR.](#@Specification_of_the_case_where_burn_type_is_LBR._1)
    -  [Specification of the case where burn type is not LBR.](#@Specification_of_the_case_where_burn_type_is_not_LBR._2)
-  [Module Specification](#@Module_Specification_3)
    -  [Initialization](#@Initialization_4)
    -  [Helper Function](#@Helper_Function_5)


<pre><code><b>use</b> <a href="Coin1.md#0x1_Coin1">0x1::Coin1</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="LBR.md#0x1_LBR">0x1::LBR</a>;
<b>use</b> <a href="Libra.md#0x1_Libra">0x1::Libra</a>;
<b>use</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp">0x1::LibraTimestamp</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
</code></pre>



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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_TransactionFee_ETRANSACTION_FEE"></a>

A <code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code> resource is not in the required state


<pre><code><b>const</b> <a href="TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">ETRANSACTION_FEE</a>: u64 = 0;
</code></pre>



<a name="0x1_TransactionFee_initialize"></a>

## Function `initialize`

Called in genesis. Sets up the needed resources to collect transaction fees from the
<code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code> resource with the TreasuryCompliance account.


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_initialize">initialize</a>(tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_initialize">initialize</a>(
    tc_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    // accept fees in all the currencies
    <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(tc_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="TransactionFee.md#0x1_TransactionFee_AddTxnFeeCurrencyAbortsIf">AddTxnFeeCurrencyAbortsIf</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;;
<b>ensures</b> <a href="TransactionFee.md#0x1_TransactionFee_is_initialized">is_initialized</a>();
<b>ensures</b> <a href="TransactionFee.md#0x1_TransactionFee_spec_transaction_fee">spec_transaction_fee</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;().balance.value == 0;
</code></pre>




<a name="0x1_TransactionFee_AddTxnFeeCurrencyAbortsIf"></a>


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_AddTxnFeeCurrencyAbortsIf">AddTxnFeeCurrencyAbortsIf</a>&lt;CoinType&gt; {
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;CoinType&gt;;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>())
        <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
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
    <b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>())
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
    <a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;()
}
</code></pre>



</details>

<a name="0x1_TransactionFee_add_txn_fee_currency"></a>

## Function `add_txn_fee_currency`

Sets ups the needed transaction fee state for a given <code>CoinType</code> currency by
(1) configuring <code>tc_account</code> to accept <code>CoinType</code>
(2) publishing a wrapper of the <code>Preburn&lt;CoinType&gt;</code> resource under <code>tc_account</code>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(tc_account: &signer) {
    <a href="Libra.md#0x1_Libra_assert_is_currency">Libra::assert_is_currency</a>&lt;CoinType&gt;();
    <b>assert</b>(
        !<a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;(),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">ETRANSACTION_FEE</a>)
    );
    move_to(
        tc_account,
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
        <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>(),
    );
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> fees.balance, coin)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>aborts_if</b> !<a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;() <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
<a name="0x1_TransactionFee_fees$8"></a>
<b>let</b> fees = <a href="TransactionFee.md#0x1_TransactionFee_spec_transaction_fee">spec_transaction_fee</a>&lt;CoinType&gt;().balance;
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
    <b>let</b> tc_address = <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>();
    <b>if</b> (<a href="LBR.md#0x1_LBR_is_lbr">LBR::is_lbr</a>&lt;CoinType&gt;()) {
        // TODO: Once the composition of <a href="LBR.md#0x1_LBR">LBR</a> is determined fill this in <b>to</b>
        // <b>unpack</b> and burn the backing coins of the <a href="LBR.md#0x1_LBR">LBR</a> coin.
        <b>abort</b> <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">ETRANSACTION_FEE</a>)
    } <b>else</b> {
        // extract fees
        <b>let</b> fees = borrow_global_mut&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(tc_address);
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


Must abort if the account does not have the TreasuryCompliance role [[H3]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>aborts_if</b> !<a href="TransactionFee.md#0x1_TransactionFee_is_coin_initialized">is_coin_initialized</a>&lt;CoinType&gt;() <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
<b>include</b> <b>if</b> (<a href="LBR.md#0x1_LBR_spec_is_lbr">LBR::spec_is_lbr</a>&lt;CoinType&gt;()) <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesLBR">BurnFeesLBR</a> <b>else</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesNotLBR">BurnFeesNotLBR</a>&lt;CoinType&gt;;
</code></pre>


The correct amount of fees is burnt and subtracted from market cap.


<pre><code><b>ensures</b> <a href="Libra.md#0x1_Libra_spec_market_cap">Libra::spec_market_cap</a>&lt;CoinType&gt;()
    == <b>old</b>(<a href="Libra.md#0x1_Libra_spec_market_cap">Libra::spec_market_cap</a>&lt;CoinType&gt;()) - <b>old</b>(<a href="TransactionFee.md#0x1_TransactionFee_spec_transaction_fee">spec_transaction_fee</a>&lt;CoinType&gt;().balance.value);
</code></pre>


All the fees is burnt so the balance becomes 0.


<pre><code><b>ensures</b> <a href="TransactionFee.md#0x1_TransactionFee_spec_transaction_fee">spec_transaction_fee</a>&lt;CoinType&gt;().balance.value == 0;
</code></pre>


STUB: To be filled in at a later date once the makeup of the LBR has been determined.


<a name="@Specification_of_the_case_where_burn_type_is_LBR._1"></a>

### Specification of the case where burn type is LBR.



<a name="0x1_TransactionFee_BurnFeesLBR"></a>


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesLBR">BurnFeesLBR</a> {
    tc_account: signer;
    <b>aborts_if</b> <b>true</b> <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
}
</code></pre>



<a name="@Specification_of_the_case_where_burn_type_is_not_LBR._2"></a>

### Specification of the case where burn type is not LBR.



<a name="0x1_TransactionFee_BurnFeesNotLBR"></a>


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesNotLBR">BurnFeesNotLBR</a>&lt;CoinType&gt; {
    tc_account: signer;
}
</code></pre>


Must abort if the account does not have BurnCapability [[H3]][PERMISSION].


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesNotLBR">BurnFeesNotLBR</a>&lt;CoinType&gt; {
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoBurnCapability">Libra::AbortsIfNoBurnCapability</a>&lt;CoinType&gt;{account: tc_account};
    <a name="0x1_TransactionFee_fees$7"></a>
    <b>let</b> fees = <a href="TransactionFee.md#0x1_TransactionFee_spec_transaction_fee">spec_transaction_fee</a>&lt;CoinType&gt;();
    <b>include</b> <a href="Libra.md#0x1_Libra_BurnNowAbortsIf">Libra::BurnNowAbortsIf</a>&lt;CoinType&gt;{coin: fees.balance, preburn: fees.preburn};
}
</code></pre>


tc_account retrieves BurnCapability [[H3]][PERMISSION].
BurnCapability is not transferrable [[J3]][PERMISSION].


<pre><code><b>schema</b> <a href="TransactionFee.md#0x1_TransactionFee_BurnFeesNotLBR">BurnFeesNotLBR</a>&lt;CoinType&gt; {
    <b>ensures</b> <b>exists</b>&lt;<a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(tc_account));
}
</code></pre>



</details>

<a name="@Module_Specification_3"></a>

## Module Specification



<a name="@Initialization_4"></a>

### Initialization


If time has started ticking, then <code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code> resources have been initialized.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="TransactionFee.md#0x1_TransactionFee_is_initialized">is_initialized</a>();
</code></pre>



<a name="@Helper_Function_5"></a>

### Helper Function



<a name="0x1_TransactionFee_spec_transaction_fee"></a>


<pre><code><b>define</b> <a href="TransactionFee.md#0x1_TransactionFee_spec_transaction_fee">spec_transaction_fee</a>&lt;CoinType&gt;(): <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt; {
   borrow_global&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;CoinType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>())
}
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/libra/lip/blob/master/lips/lip-2.md
[ROLE]: https://github.com/libra/lip/blob/master/lips/lip-2.md#roles
[PERMISSION]: https://github.com/libra/lip/blob/master/lips/lip-2.md#permissions
