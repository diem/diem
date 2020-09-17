
<a name="SCRIPT"></a>

# Script `burn_txn_fees.move`

### Table of Contents

-  [Function `burn_txn_fees`](#SCRIPT_burn_txn_fees)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
        -  [Events](#SCRIPT_@Events)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_burn_txn_fees"></a>

## Function `burn_txn_fees`


<a name="SCRIPT_@Summary"></a>

### Summary

Burns the transaction fees collected in the <code>CoinType</code> currency so that the
Libra association may reclaim the backing coins off-chain. May only be sent
by the Treasury Compliance account.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Burns the transaction fees collected in <code>CoinType</code> so that the
association may reclaim the backing coins. Once this transaction has executed
successfully all transaction fees that will have been collected in
<code>CoinType</code> since the last time this script was called with that specific
currency. Both <code>balance</code> and <code>preburn</code> fields in the
<code><a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_TransactionFee">TransactionFee::TransactionFee</a>&lt;CoinType&gt;</code> resource published under the <code>0xB1E55ED</code>
account address will have a value of 0 after the successful execution of this script.


<a name="SCRIPT_@Events"></a>

#### Events

The successful execution of this transaction will emit a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnEvent">Libra::BurnEvent</a></code> on the event handle
held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;CoinType&gt;</code> resource's <code>burn_events</code> published under
<code>0xA550C18</code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name         | Type      | Description                                                                                                                                         |
| ------       | ------    | -------------                                                                                                                                       |
| <code>CoinType</code>   | Type      | The Move type for the <code>CoinType</code> being added to the sending account of the transaction. <code>CoinType</code> must be an already-registered currency on-chain. |
| <code>tc_account</code> | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.                                           |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                          | Description                                                 |
| ----------------           | --------------                        | -------------                                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code> | The sending account is not the Treasury Compliance account. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_ETRANSACTION_FEE">TransactionFee::ETRANSACTION_FEE</a></code>    | <code>CoinType</code> is not an accepted transaction fee currency.     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECOIN">Libra::ECOIN</a></code>                        | The collected fees in <code>CoinType</code> are zero.                  |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::burn</code>
* <code>Script::cancel_burn</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_burn_txn_fees">burn_txn_fees</a>&lt;CoinType&gt;(tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_burn_txn_fees">burn_txn_fees</a>&lt;CoinType&gt;(tc_account: &signer) {
    <a href="../../modules/doc/TransactionFee.md#0x1_TransactionFee_burn_fees">TransactionFee::burn_fees</a>&lt;CoinType&gt;(tc_account);
}
</code></pre>



</details>
