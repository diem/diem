
<a name="SCRIPT"></a>

# Script `preburn.move`

### Table of Contents

-  [Function `preburn`](#SCRIPT_preburn)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
        -  [Events](#SCRIPT_@Events)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)
-  [Specification](#SCRIPT_Specification)
    -  [Function `preburn`](#SCRIPT_Specification_preburn)



<a name="SCRIPT_preburn"></a>

## Function `preburn`


<a name="SCRIPT_@Summary"></a>

### Summary

Moves a specified number of coins in a given currency from the account's
balance to its preburn area after which the coins may be burned. This
transaction may be sent by any account that holds a balance and preburn area
in the specified currency.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Moves the specified <code>amount</code> of coins in <code>Token</code> currency from the sending <code>account</code>'s
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;</code> to the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> published under the same
<code>account</code>. <code>account</code> must have both of these resources published under it at the start of this
transaction in order for it to execute successfully.


<a name="SCRIPT_@Events"></a>

#### Events

Successful execution of this script emits two events:
* <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a> </code> on <code>account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code>
handle with the <code>payee</code> and <code>payer</code> fields being <code>account</code>'s address; and
* A <code><a href="../../modules/doc/Libra.md#0x1_Libra_PreburnEvent">Libra::PreburnEvent</a></code> with <code>Token</code>'s currency code on the
<code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token</code>'s <code>preburn_events</code> handle for <code>Token</code> and with
<code>preburn_address</code> set to <code>account</code>'s address.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name      | Type      | Description                                                                                                                      |
| ------    | ------    | -------------                                                                                                                    |
| <code>Token</code>   | Type      | The Move type for the <code>Token</code> currency being moved to the preburn area. <code>Token</code> must be an already-registered currency on-chain. |
| <code>account</code> | <code>&signer</code> | The signer reference of the sending account.                                                                                     |
| <code>amount</code>  | <code>u64</code>     | The amount in <code>Token</code> to be moved to the preburn area.                                                                           |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category           | Error Reason                                             | Description                                                                             |
| ----------------         | --------------                                           | -------------                                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>  | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                                  | The <code>Token</code> is not a registered currency on-chain.                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>  | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a></code> | The withdrawal capability for <code>account</code> has already been extracted.                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">LibraAccount::EINSUFFICIENT_BALANCE</a></code>                    | <code>amount</code> is greater than <code>payer</code>'s balance in <code>Token</code>.                                  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>  | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>              | <code>account</code> doesn't hold a balance in <code>Token</code>.                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>  | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN">Libra::EPREBURN</a></code>                                        | <code>account</code> doesn't have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under it.           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>  | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN_OCCUPIED">Libra::EPREBURN_OCCUPIED</a></code>                               | The <code>value</code> field in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource under the sender is non-zero. |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::cancel_burn</code>
* <code>Script::burn</code>
* <code>Script::burn_txn_fees</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_preburn">preburn</a>&lt;Token&gt;(account: &signer, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_preburn">preburn</a>&lt;Token&gt;(account: &signer, amount: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_preburn">LibraAccount::preburn</a>&lt;Token&gt;(account, &withdraw_cap, amount);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap);
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_preburn"></a>

### Function `preburn`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_preburn">preburn</a>&lt;Token&gt;(account: &signer, amount: u64)
</code></pre>




<pre><code>pragma verify;
<a name="SCRIPT_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<a name="SCRIPT_cap$2"></a>
<b>let</b> cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">LibraAccount::spec_get_withdraw_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">LibraAccount::ExtractWithdrawCapAbortsIf</a>{sender_addr: account_addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_PreburnAbortsIf">LibraAccount::PreburnAbortsIf</a>&lt;Token&gt;{dd: account, cap: cap};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_PreburnEnsures">LibraAccount::PreburnEnsures</a>&lt;Token&gt;{dd_addr: account_addr, payer: account_addr};
</code></pre>
