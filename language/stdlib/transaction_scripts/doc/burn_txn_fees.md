
<a name="burn_txn_fees"></a>

# Script `burn_txn_fees`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
    -  [Events](#@Events_2)
-  [Parameters](#@Parameters_3)
-  [Common Abort Conditions](#@Common_Abort_Conditions_4)
-  [Related Scripts](#@Related_Scripts_5)


<a name="@Summary_0"></a>

## Summary

Burns the transaction fees collected in the <code>CoinType</code> currency so that the
Libra association may reclaim the backing coins off-chain. May only be sent
by the Treasury Compliance account.


<a name="@Technical_Description_1"></a>

## Technical Description

Burns the transaction fees collected in <code>CoinType</code> so that the
association may reclaim the backing coins. Once this transaction has executed
successfully all transaction fees that will have been collected in
<code>CoinType</code> since the last time this script was called with that specific
currency. Both <code>balance</code> and <code><a href="preburn.md#preburn">preburn</a></code> fields in the
<code><a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;CoinType&gt;</code> resource published under the <code>0xB1E55ED</code>
account address will have a value of 0 after the successful execution of this script.


<a name="@Events_2"></a>

### Events

The successful execution of this transaction will emit a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnEvent">Libra::BurnEvent</a></code> on the event handle
held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;CoinType&gt;</code> resource's <code>burn_events</code> published under
<code>0xA550C18</code>.


<a name="@Parameters_3"></a>

## Parameters

| Name         | Type      | Description                                                                                                                                         |
| ------       | ------    | -------------                                                                                                                                       |
| <code>CoinType</code>   | Type      | The Move type for the <code>CoinType</code> being added to the sending account of the transaction. <code>CoinType</code> must be an already-registered currency on-chain. |
| <code>tc_account</code> | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.                                           |


<a name="@Common_Abort_Conditions_4"></a>

## Common Abort Conditions

| Error Category             | Error Reason                          | Description                                                 |
| ----------------           | --------------                        | -------------                                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code> | The sending account is not the Treasury Compliance account. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">TransactionFee::ETRANSACTION_FEE</a></code>    | <code>CoinType</code> is not an accepted transaction fee currency.     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECOIN">Libra::ECOIN</a></code>                        | The collected fees in <code>CoinType</code> are zero.                  |


<a name="@Related_Scripts_5"></a>

## Related Scripts

* <code><a href="burn.md#burn">Script::burn</a></code>
* <code><a href="cancel_burn.md#cancel_burn">Script::cancel_burn</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="burn_txn_fees.md#burn_txn_fees">burn_txn_fees</a>&lt;CoinType&gt;(tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="burn_txn_fees.md#burn_txn_fees">burn_txn_fees</a>&lt;CoinType&gt;(tc_account: &signer) {
    <a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_burn_fees">TransactionFee::burn_fees</a>&lt;CoinType&gt;(tc_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma verify = <b>false</b>;
</code></pre>



</details>
