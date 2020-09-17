
<a name="SCRIPT"></a>

# Script `cancel_burn.move`

### Table of Contents

-  [Function `cancel_burn`](#SCRIPT_cancel_burn)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
        -  [Events](#SCRIPT_@Events)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_cancel_burn"></a>

## Function `cancel_burn`


<a name="SCRIPT_@Summary"></a>

### Summary

Cancels and returns all coins held in the preburn area under
<code>preburn_address</code> and returns the funds to the <code>preburn_address</code>'s balance.
Can only be successfully sent by an account with Treasury Compliance role.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Cancels and returns all coins held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource under the <code>preburn_address</code> and
return the funds to the <code>preburn_address</code> account's <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;</code>.
The transaction must be sent by an <code>account</code> with a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;</code>
resource published under it. The account at <code>preburn_address</code> must have a
<code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under it, and its value must be nonzero. The transaction removes
the entire balance held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource, and returns it back to the account's
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;</code> under <code>preburn_address</code>. Due to this, the account at
<code>preburn_address</code> must already have a balance in the <code>Token</code> currency published
before this script is called otherwise the transaction will fail.


<a name="SCRIPT_@Events"></a>

#### Events

The successful execution of this transaction will emit:
* A <code><a href="../../modules/doc/Libra.md#0x1_Libra_CancelBurnEvent">Libra::CancelBurnEvent</a></code> on the event handle held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token&gt;</code>
resource's <code>burn_events</code> published under <code>0xA550C18</code>.
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> on the <code>preburn_address</code>'s
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>received_events</code> event handle with both the <code>payer</code> and <code>payee</code>
being <code>preburn_address</code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name              | Type      | Description                                                                                                                          |
| ------            | ------    | -------------                                                                                                                        |
| <code>Token</code>           | Type      | The Move type for the <code>Token</code> currenty that burning is being cancelled for. <code>Token</code> must be an already-registered currency on-chain. |
| <code>account</code>         | <code>&signer</code> | The signer reference of the sending account of this transaction, must have a burn capability for <code>Token</code> published under it.         |
| <code>preburn_address</code> | <code>address</code> | The address where the coins to-be-burned are currently held.                                                                         |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category                | Error Reason                                     | Description                                                                                           |
| ----------------              | --------------                                   | -------------                                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EBURN_CAPABILITY">Libra::EBURN_CAPABILITY</a></code>                        | The sending <code>account</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;</code> published under it.              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN">Libra::EPREBURN</a></code>                                | The account at <code>preburn_address</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under it. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                          | The specified <code>Token</code> is not a registered currency on-chain.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ECOIN_DEPOSIT_IS_ZERO">LibraAccount::ECOIN_DEPOSIT_IS_ZERO</a></code>            | The value held in the preburn resource was zero.                                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code> | The account at <code>preburn_address</code> doesn't have a balance resource for <code>Token</code>.                         |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::burn_txn_fees</code>
* <code>Script::burn</code>
* <code>Script::preburn</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_cancel_burn">cancel_burn</a>&lt;Token&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_cancel_burn">cancel_burn</a>&lt;Token&gt;(account: &signer, preburn_address: address) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_cancel_burn">LibraAccount::cancel_burn</a>&lt;Token&gt;(account, preburn_address)
}
</code></pre>



</details>
