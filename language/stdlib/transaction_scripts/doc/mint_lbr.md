
<a name="SCRIPT"></a>

# Script `mint_lbr.move`

### Table of Contents

-  [Function `mint_lbr`](#SCRIPT_mint_lbr)
    -  [Events](#SCRIPT_@Events)
    -  [Abort Conditions](#SCRIPT_@Abort_Conditions)
        -  [Aborts Caused by Invalid Account State](#SCRIPT_@Aborts_Caused_by_Invalid_Account_State)
        -  [Aborts Caused by Invalid LBR Minting State](#SCRIPT_@Aborts_Caused_by_Invalid_LBR_Minting_State)
        -  [Other Aborts](#SCRIPT_@Other_Aborts)
    -  [Post Conditions](#SCRIPT_@Post_Conditions)
        -  [Changed States](#SCRIPT_@Changed_States)
        -  [Unchanged States](#SCRIPT_@Unchanged_States)
-  [Specification](#SCRIPT_Specification)
    -  [Function `mint_lbr`](#SCRIPT_Specification_mint_lbr)



<a name="SCRIPT_mint_lbr"></a>

## Function `mint_lbr`

Mint
<code>amount_lbr</code> LBR from the sending account's constituent coins and deposits the
resulting LBR into the sending account.


<a name="SCRIPT_@Events"></a>

### Events

When this script executes without aborting, it emits three events:
<code>SentPaymentEvent { amount_coin1, currency_code = <a href="../../modules/doc/Coin1.md#0x1_Coin1">Coin1</a>, address_of(account), metadata = x"" }</code>
<code>SentPaymentEvent { amount_coin2, currency_code = <a href="../../modules/doc/Coin2.md#0x1_Coin2">Coin2</a>, address_of(account), metadata = x"" }</code>
on
<code>account</code>'s
<code>LibraAccount::sent_events</code> handle where
<code>amount_coin1</code> and
<code>amount_coin2</code>
are the components amounts of
<code>amount_lbr</code> LBR, and
<code>ReceivedPaymentEvent { amount_lbr, currency_code = <a href="../../modules/doc/LBR.md#0x1_LBR">LBR</a>, address_of(account), metadata = x"" }</code>
on
<code>account</code>'s
<code>LibraAccount::received_events</code> handle.


<a name="SCRIPT_@Abort_Conditions"></a>

### Abort Conditions

> TODO(emmazzz): the documentation below documents the reasons of abort, instead of the categories.
> We might want to discuss about what the best approach is here.
The following abort conditions have been formally specified and verified. See spec schema
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBRAbortsIf">LibraAccount::StapleLBRAbortsIf</a></code> for the formal specifications.


<a name="SCRIPT_@Aborts_Caused_by_Invalid_Account_State"></a>

#### Aborts Caused by Invalid Account State

* Aborts with
<code>LibraAccount::EINSUFFICIENT_BALANCE</code> if
<code>amount_coin1</code> is greater than sending
<code>account</code>'s balance in
<code><a href="../../modules/doc/Coin1.md#0x1_Coin1">Coin1</a></code> or if
<code>amount_coin2</code> is greater than sending
<code>account</code>'s balance in
<code><a href="../../modules/doc/Coin2.md#0x1_Coin2">Coin2</a></code>.
* Aborts with
<code>LibraAccount::ECOIN_DEPOSIT_IS_ZERO</code> if
<code>amount_lbr</code> is zero.
* Aborts with
<code>LibraAccount::EPAYEE_DOES_NOT_EXIST</code> if no LibraAccount exists at the address of
<code>account</code>.
* Aborts with
<code>LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</code> if LibraAccount exists at
<code>account</code>,
but it does not accept payments in LBR.


<a name="SCRIPT_@Aborts_Caused_by_Invalid_LBR_Minting_State"></a>

#### Aborts Caused by Invalid LBR Minting State

* Aborts with
<code>Libra::EMINTING_NOT_ALLOWED</code> if minting LBR is not allowed according to the CurrencyInfo<LBR>
stored at
<code>CURRENCY_INFO_ADDRESS</code>.
* Aborts with
<code>Libra::ECURRENCY_INFO</code> if the total value of LBR would reach
<code>MAX_U128</code> after
<code>amount_lbr</code>
LBR is minted.


<a name="SCRIPT_@Other_Aborts"></a>

#### Other Aborts

These aborts should only happen when
<code>account</code> has account limit restrictions or
has been frozen by Libra administrators.
* Aborts with
<code>LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS</code> if
<code>account</code> has exceeded their daily
withdrawal limits for Coin1 or Coin2.
* Aborts with
<code>LibraAccount::EDEPOSIT_EXCEEDS_LIMITS</code> if
<code>account</code> has exceeded their daily
deposit limits for LBR.
* Aborts with
<code>LibraAccount::EACCOUNT_FROZEN</code> if
<code>account</code> is frozen.


<a name="SCRIPT_@Post_Conditions"></a>

### Post Conditions

The following post conditions have been formally specified and verified. See spec schema
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBREnsures">LibraAccount::StapleLBREnsures</a></code> for the formal specifications.


<a name="SCRIPT_@Changed_States"></a>

#### Changed States

* The reserve backing for Coin1 and Coin2 increase by the right amounts as specified by the component ratio.
* Coin1 and Coin2 balances at the address of sending
<code>account</code> decrease by the right amounts as specified by
the component ratio.
* The total value of LBR increases by
<code>amount_lbr</code>.
* LBR balance at the address of sending
<code>account</code> increases by
<code>amount_lbr</code>.


<a name="SCRIPT_@Unchanged_States"></a>

#### Unchanged States

* The total values of Coin1 and Coin2 stay the same.


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



> TODO: timeout due to FixedPoint32 flakiness.


<pre><code>pragma verify = <b>false</b>;
<a name="SCRIPT_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<a name="SCRIPT_cap$2"></a>
<b>let</b> cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">LibraAccount::spec_get_withdraw_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">LibraAccount::ExtractWithdrawCapAbortsIf</a>{sender_addr: account_addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBRAbortsIf">LibraAccount::StapleLBRAbortsIf</a>{cap: cap};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBREnsures">LibraAccount::StapleLBREnsures</a>{cap: cap};
</code></pre>
