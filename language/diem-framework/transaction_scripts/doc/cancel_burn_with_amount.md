
<a name="cancel_burn_with_amount"></a>

# Script `cancel_burn_with_amount`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
    -  [Events](#@Events_2)
-  [Parameters](#@Parameters_3)
-  [Common Abort Conditions](#@Common_Abort_Conditions_4)
-  [Related Scripts](#@Related_Scripts_5)


<pre><code><b>use</b> <a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount">0x1::DiemAccount</a>;
</code></pre>



<a name="@Summary_0"></a>

## Summary

Cancels and returns the coins held in the preburn area under
<code>preburn_address</code>, which are equal to the <code>amount</code> specified in the transaction. Finds the first preburn
resource with the matching amount and returns the funds to the <code>preburn_address</code>'s balance.
Can only be successfully sent by an account with Treasury Compliance role.


<a name="@Technical_Description_1"></a>

## Technical Description

Cancels and returns all coins held in the <code><a href="../../modules/doc/Diem.md#0x1_Diem_Preburn">Diem::Preburn</a>&lt;Token&gt;</code> resource under the <code>preburn_address</code> and
return the funds to the <code>preburn_address</code> account's <code><a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_Balance">DiemAccount::Balance</a>&lt;Token&gt;</code>.
The transaction must be sent by an <code>account</code> with a <code><a href="../../modules/doc/Diem.md#0x1_Diem_BurnCapability">Diem::BurnCapability</a>&lt;Token&gt;</code>
resource published under it. The account at <code>preburn_address</code> must have a
<code><a href="../../modules/doc/Diem.md#0x1_Diem_Preburn">Diem::Preburn</a>&lt;Token&gt;</code> resource published under it, and its value must be nonzero. The transaction removes
the entire balance held in the <code><a href="../../modules/doc/Diem.md#0x1_Diem_Preburn">Diem::Preburn</a>&lt;Token&gt;</code> resource, and returns it back to the account's
<code><a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_Balance">DiemAccount::Balance</a>&lt;Token&gt;</code> under <code>preburn_address</code>. Due to this, the account at
<code>preburn_address</code> must already have a balance in the <code>Token</code> currency published
before this script is called otherwise the transaction will fail.


<a name="@Events_2"></a>

### Events

The successful execution of this transaction will emit:
* A <code><a href="../../modules/doc/Diem.md#0x1_Diem_CancelBurnEvent">Diem::CancelBurnEvent</a></code> on the event handle held in the <code><a href="../../modules/doc/Diem.md#0x1_Diem_CurrencyInfo">Diem::CurrencyInfo</a>&lt;Token&gt;</code>
resource's <code>burn_events</code> published under <code>0xA550C18</code>.
* A <code><a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_ReceivedPaymentEvent">DiemAccount::ReceivedPaymentEvent</a></code> on the <code>preburn_address</code>'s
<code><a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_DiemAccount">DiemAccount::DiemAccount</a></code> <code>received_events</code> event handle with both the <code>payer</code> and <code>payee</code>
being <code>preburn_address</code>.


<a name="@Parameters_3"></a>

## Parameters

| Name              | Type      | Description                                                                                                                          |
| ------            | ------    | -------------                                                                                                                        |
| <code>Token</code>           | Type      | The Move type for the <code>Token</code> currenty that burning is being cancelled for. <code>Token</code> must be an already-registered currency on-chain. |
| <code>account</code>         | <code>&signer</code> | The signer reference of the sending account of this transaction, must have a burn capability for <code>Token</code> published under it.         |
| <code>preburn_address</code> | <code>address</code> | The address where the coins to-be-burned are currently held.                                                                         |
| <code>amount</code>          | <code>u64</code>     | The amount to be cancelled.                                                                                                          |


<a name="@Common_Abort_Conditions_4"></a>

## Common Abort Conditions

| Error Category                | Error Reason                                     | Description                                                                                                                         |
| ----------------              | --------------                                   | -------------                                                                                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a></code> | <code><a href="../../modules/doc/Diem.md#0x1_Diem_EBURN_CAPABILITY">Diem::EBURN_CAPABILITY</a></code>                         | The sending <code>account</code> does not have a <code><a href="../../modules/doc/Diem.md#0x1_Diem_BurnCapability">Diem::BurnCapability</a>&lt;Token&gt;</code> published under it.                                             |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>       | <code><a href="../../modules/doc/Diem.md#0x1_Diem_EPREBURN_NOT_FOUND">Diem::EPREBURN_NOT_FOUND</a></code>                       | The <code><a href="../../modules/doc/Diem.md#0x1_Diem_PreburnQueue">Diem::PreburnQueue</a>&lt;Token&gt;</code> resource under <code>preburn_address</code> does not contain a preburn request with a value matching <code>amount</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Diem.md#0x1_Diem_EPREBURN_QUEUE">Diem::EPREBURN_QUEUE</a></code>                           | The account at <code>preburn_address</code> does not have a <code><a href="../../modules/doc/Diem.md#0x1_Diem_PreburnQueue">Diem::PreburnQueue</a>&lt;Token&gt;</code> resource published under it.                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Diem.md#0x1_Diem_ECURRENCY_INFO">Diem::ECURRENCY_INFO</a></code>                           | The specified <code>Token</code> is not a registered currency on-chain.                                                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">DiemAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code>  | The account at <code>preburn_address</code> doesn't have a balance resource for <code>Token</code>.                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>      | <code><a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_EDEPOSIT_EXCEEDS_LIMITS">DiemAccount::EDEPOSIT_EXCEEDS_LIMITS</a></code>           | The depositing of the funds held in the prebun area would exceed the <code>account</code>'s account limits.                                    |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>       | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_EPAYEE_COMPLIANCE_KEY_NOT_SET">DualAttestation::EPAYEE_COMPLIANCE_KEY_NOT_SET</a></code> | The <code>account</code> does not have a compliance key set on it but dual attestion checking was performed.                                   |


<a name="@Related_Scripts_5"></a>

## Related Scripts

* <code><a href="transaction_script_documentation.md#burn_txn_fees">Script::burn_txn_fees</a></code>
* <code>Script::burn</code>
* <code><a href="transaction_script_documentation.md#preburn">Script::preburn</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="cancel_burn_with_amount.md#cancel_burn_with_amount">cancel_burn_with_amount</a>&lt;Token&gt;(account: &signer, preburn_address: address, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="cancel_burn_with_amount.md#cancel_burn_with_amount">cancel_burn_with_amount</a>&lt;Token&gt;(account: &signer, preburn_address: address, amount: u64) {
    <a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_cancel_burn">DiemAccount::cancel_burn</a>&lt;Token&gt;(account, preburn_address, amount)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_TransactionChecks">DiemAccount::TransactionChecks</a>{sender: account};
<b>include</b> <a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_CancelBurnAbortsIf">DiemAccount::CancelBurnAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="../../modules/doc/Diem.md#0x1_Diem_CancelBurnWithCapEnsures">Diem::CancelBurnWithCapEnsures</a>&lt;Token&gt;;
<b>include</b> <a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_DepositEnsures">DiemAccount::DepositEnsures</a>&lt;Token&gt;{payee: preburn_address};
<a name="cancel_burn_with_amount_total_preburn_value$1"></a>
<b>let</b> total_preburn_value = <b>global</b>&lt;<a href="../../modules/doc/Diem.md#0x1_Diem_CurrencyInfo">Diem::CurrencyInfo</a>&lt;Token&gt;&gt;(
    <a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()
).preburn_value;
<a name="cancel_burn_with_amount_balance_at_addr$2"></a>
<b>let</b> balance_at_addr = <a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_balance">DiemAccount::balance</a>&lt;Token&gt;(preburn_address);
</code></pre>


The total value of preburn for <code>Token</code> should decrease by the preburned amount.


<pre><code><b>ensures</b> total_preburn_value == <b>old</b>(total_preburn_value) - amount;
</code></pre>


The balance of <code>Token</code> at <code>preburn_address</code> should increase by the preburned amount.


<pre><code><b>ensures</b> balance_at_addr == <b>old</b>(balance_at_addr) + amount;
<b>include</b> <a href="../../modules/doc/Diem.md#0x1_Diem_CancelBurnWithCapEmits">Diem::CancelBurnWithCapEmits</a>&lt;Token&gt;;
<b>include</b> <a href="../../modules/doc/DiemAccount.md#0x1_DiemAccount_DepositEmits">DiemAccount::DepositEmits</a>&lt;Token&gt;{
    payer: preburn_address,
    payee: preburn_address,
    amount: amount,
    metadata: x""
};
<b>aborts_with</b> [check]
    <a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>,
    <a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
</code></pre>


**Access Control:**
Only the account with the burn capability can cancel burning [[H3]][PERMISSION].


<pre><code><b>include</b> <a href="../../modules/doc/Diem.md#0x1_Diem_AbortsIfNoBurnCapability">Diem::AbortsIfNoBurnCapability</a>&lt;Token&gt;{account: account};
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
