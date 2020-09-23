
<a name="unmint_lbr"></a>

# Script `unmint_lbr`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
    -  [Events](#@Events_2)
-  [Parameters](#@Parameters_3)
-  [Common Abort Conditions](#@Common_Abort_Conditions_4)
-  [Related Scripts](#@Related_Scripts_5)


<a name="@Summary_0"></a>

## Summary

Withdraws a specified amount of LBR from the transaction sender's account, and unstaples the
withdrawn LBR into its constituent coins. Deposits each of the constituent coins to the
transaction sender's balances. Any account that can hold balances that has the correct balances
may send this transaction.


<a name="@Technical_Description_1"></a>

## Technical Description

Withdraws <code>amount_lbr</code> LBR coins from the <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;<a href="../../modules/doc/LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;</code> balance held under
<code>account</code>. Withdraws the backing coins for the LBR coins from the on-chain reserve in the
<code><a href="../../modules/doc/LBR.md#0x1_LBR_Reserve">LBR::Reserve</a></code> resource published under <code>0xA550C18</code>. It then deposits each of the backing coins
into balance resources published under <code>account</code>.


<a name="@Events_2"></a>

### Events

Successful execution of this transaction will emit two <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code>s. One
for each constituent currency that is unstapled and returned to the sending <code>account</code>'s
balances.


<a name="@Parameters_3"></a>

## Parameters

| Name         | Type      | Description                                                     |
| ------       | ------    | -------------                                                   |
| <code>account</code>    | <code>&signer</code> | The signer reference of the sending account of the transaction. |
| <code>amount_lbr</code> | <code>u64</code>     | The amount of microlibra to unstaple.                           |


<a name="@Common_Abort_Conditions_4"></a>

## Common Abort Conditions

| Error Category             | Error Reason                                             | Description                                                                               |
| ----------------           | --------------                                           | -------------                                                                             |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a></code> | The <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a></code> for <code>account</code> has previously been extracted.       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>              | <code>account</code> doesn't have a balance in LBR.                                                  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">LibraAccount::EINSUFFICIENT_BALANCE</a></code>                    | <code>amount_lbr</code> is greater than the balance of LBR in <code>account</code>.                             |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECOIN">Libra::ECOIN</a></code>                                           | <code>amount_lbr</code> is zero.                                                                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS">LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS</a></code>               | <code>account</code> has exceeded its daily withdrawal limits for LBR.                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS">LibraAccount::EDEPOSIT_EXCEEDS_LIMITS</a></code>                  | <code>account</code> has exceeded its daily deposit limits for one of the backing currencies of LBR. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code>         | <code>account</code> doesn't hold a balance in one or both of the backing currencies of LBR.         |


<a name="@Related_Scripts_5"></a>

## Related Scripts

* <code><a href="overview.md#mint_lbr">Script::mint_lbr</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="unmint_lbr.md#unmint_lbr">unmint_lbr</a>(account: &signer, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="unmint_lbr.md#unmint_lbr">unmint_lbr</a>(account: &signer, amount_lbr: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_unstaple_lbr">LibraAccount::unstaple_lbr</a>(&withdraw_cap, amount_lbr);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap);
}
</code></pre>



</details>
