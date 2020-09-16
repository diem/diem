
<a name="SCRIPT"></a>

# Script `mint_lbr.move`

### Table of Contents

-  [Function `mint_lbr`](#SCRIPT_mint_lbr)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
        -  [Events](#SCRIPT_@Events)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)
-  [Specification](#SCRIPT_Specification)
    -  [Function `mint_lbr`](#SCRIPT_Specification_mint_lbr)



<a name="SCRIPT_mint_lbr"></a>

## Function `mint_lbr`


<a name="SCRIPT_@Summary"></a>

### Summary

Mints LBR from the sending account's constituent coins by depositing in the
on-chain LBR reserve. Deposits the newly-minted LBR into the sending
account. Can be sent by any account that can hold balances for the constituent
currencies for LBR and LBR.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Mints <code>amount_lbr</code> LBR from the sending account's constituent coins and deposits the
resulting LBR into the sending account.


<a name="SCRIPT_@Events"></a>

#### Events

Successful execution of this script emits three events:
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> with the Coin1 currency code, and a
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> with the Coin2 currency code on <code>account</code>'s
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code> handle with the <code>amounts</code> for each event being the
components amounts of <code>amount_lbr</code> LBR; and
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> on <code>account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code>
<code>received_events</code> handle with the LBR currency code and amount field equal to <code>amount_lbr</code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name         | Type      | Description                                      |
| ------       | ------    | -------------                                    |
| <code>account</code>    | <code>&signer</code> | The signer reference of the sending account.     |
| <code>amount_lbr</code> | <code>u64</code>     | The amount of LBR (in microlibra) to be created. |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                     | Description                                                                      |
| ----------------           | --------------                                   | -------------                                                                    |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>      | <code>account</code> doesn't hold a balance in one of the backing currencies of LBR.        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LBR.md#0x1_LBR_EZERO_LBR_MINT_NOT_ALLOWED">LBR::EZERO_LBR_MINT_NOT_ALLOWED</a></code>                | <code>amount_lbr</code> passed in was zero.                                                 |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LBR.md#0x1_LBR_ECOIN1">LBR::ECOIN1</a></code>                                    | The amount of <code><a href="../../modules/doc/Coin1.md#0x1_Coin1">Coin1</a></code> needed for the specified LBR would exceed <code><a href="../../modules/doc/LBR.md#0x1_LBR_MAX_U64">LBR::MAX_U64</a></code>.  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LBR.md#0x1_LBR_ECOIN2">LBR::ECOIN2</a></code>                                    | The amount of <code><a href="../../modules/doc/Coin2.md#0x1_Coin2">Coin2</a></code> needed for the specified LBR would exceed <code><a href="../../modules/doc/LBR.md#0x1_LBR_MAX_U64">LBR::MAX_U64</a></code>.  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EMINTING_NOT_ALLOWED">Libra::EMINTING_NOT_ALLOWED</a></code>                    | Minting of LBR is not allowed currently.                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code> | <code>account</code> doesn't hold a balance in LBR.                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS">LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS</a></code>       | <code>account</code> has exceeded its daily withdrawal limits for the backing coins of LBR. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS">LibraAccount::EDEPOSIT_EXCEEDS_LIMITS</a></code>          | <code>account</code> has exceeded its daily deposit limits for LBR.                         |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::unmint_lbr</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_mint_lbr">mint_lbr</a>(account: &signer, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_mint_lbr">mint_lbr</a>(account: &signer, amount_lbr: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_staple_lbr">LibraAccount::staple_lbr</a>(&withdraw_cap, amount_lbr);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap)
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_mint_lbr"></a>

### Function `mint_lbr`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_mint_lbr">mint_lbr</a>(account: &signer, amount_lbr: u64)
</code></pre>




<pre><code>pragma verify = <b>true</b>;
<a name="SCRIPT_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<a name="SCRIPT_cap$2"></a>
<b>let</b> cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">LibraAccount::spec_get_withdraw_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">LibraAccount::ExtractWithdrawCapAbortsIf</a>{sender_addr: account_addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBRAbortsIf">LibraAccount::StapleLBRAbortsIf</a>{cap: cap};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBREnsures">LibraAccount::StapleLBREnsures</a>{cap: cap};
</code></pre>
