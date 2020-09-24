
<a name="@Libra_Move_Transaction_Scripts_0"></a>

# Libra Move Transaction Scripts


> TODO: this is a dummy which needs to be populated

This file contains the documentation of the Libra transaction scripts.

-  [Some Scripts](#@Some_Scripts_1)
    -  [Script <code><a href="overview.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a></code>](#peer_to_peer_with_metadata)
        -  [Summary](#@Summary_2)
        -  [Technical Description](#@Technical_Description_3)
        -  [Parameters](#@Parameters_5)
        -  [Common Abort Conditions](#@Common_Abort_Conditions_6)
        -  [Related Scripts](#@Related_Scripts_7)
    -  [Script <code><a href="overview.md#mint_lbr">mint_lbr</a></code>](#mint_lbr)
        -  [Summary](#@Summary_8)
        -  [Technical Description](#@Technical_Description_9)
        -  [Parameters](#@Parameters_11)
        -  [Common Abort Conditions](#@Common_Abort_Conditions_12)
        -  [Related Scripts](#@Related_Scripts_13)
-  [Index](#@Index_14)



<a name="@Some_Scripts_1"></a>

## Some Scripts


Some scripts are included here for demonstration purposes.


<a name="peer_to_peer_with_metadata"></a>

### Script `peer_to_peer_with_metadata`



<a name="@Summary_2"></a>

#### Summary

Transfers a given number of coins in a specified currency from one account to another.
Transfers over a specified amount defined on-chain that are between two different VASPs, or
other accounts that have opted-in will be subject to on-chain checks to ensure the receiver has
agreed to receive the coins.  This transaction can be sent by any account that can hold a
balance, and to any account that can hold a balance. Both accounts must hold balances in the
currency being transacted.


<a name="@Technical_Description_3"></a>

#### Technical Description


Transfers <code>amount</code> coins of type <code>Currency</code> from <code>payer</code> to <code>payee</code> with (optional) associated
<code>metadata</code> and an (optional) <code>metadata_signature</code> on the message
<code>metadata</code> | <code><a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(payer)</code> | <code>amount</code> | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_DOMAIN_SEPARATOR">DualAttestation::DOMAIN_SEPARATOR</a></code>.
The <code>metadata</code> and <code>metadata_signature</code> parameters are only required if <code>amount</code> >=
<code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_get_cur_microlibra_limit">DualAttestation::get_cur_microlibra_limit</a></code> LBR and <code>payer</code> and <code>payee</code> are distinct VASPs.
However, a transaction sender can opt in to dual attestation even when it is not required
(e.g., a DesignatedDealer -> VASP payment) by providing a non-empty <code>metadata_signature</code>.
Standardized <code>metadata</code> LCS format can be found in <code>libra_types::transaction::metadata::Metadata</code>.


<a name="@Events_4"></a>

##### Events

Successful execution of this script emits two events:
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> on <code>payer</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code> handle; and
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> on <code>payee</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>received_events</code> handle.


<a name="@Parameters_5"></a>

#### Parameters

| Name                 | Type         | Description                                                                                                                  |
| ------               | ------       | -------------                                                                                                                |
| <code>Currency</code>           | Type         | The Move type for the <code>Currency</code> being sent in this transaction. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>payer</code>              | <code>&signer</code>    | The signer reference of the sending account that coins are being transferred from.                                           |
| <code>payee</code>              | <code>address</code>    | The address of the account the coins are being transferred to.                                                               |
| <code>metadata</code>           | <code>vector&lt;u8&gt;</code> | Optional metadata about this payment.                                                                                        |
| <code>metadata_signature</code> | <code>vector&lt;u8&gt;</code> | Optional signature over <code>metadata</code> and payment information. See                                                              |


<a name="@Common_Abort_Conditions_6"></a>

#### Common Abort Conditions

| Error Category             | Error Reason                                     | Description                                                                                                                         |
| ----------------           | --------------                                   | -------------                                                                                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY</a></code>      | <code>payer</code> doesn't hold a balance in <code>Currency</code>.                                                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">LibraAccount::EINSUFFICIENT_BALANCE</a></code>            | <code>amount</code> is greater than <code>payer</code>'s balance in <code>Currency</code>.                                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ECOIN_DEPOSIT_IS_ZERO">LibraAccount::ECOIN_DEPOSIT_IS_ZERO</a></code>            | <code>amount</code> is zero.                                                                                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>    | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_DOES_NOT_EXIST">LibraAccount::EPAYEE_DOES_NOT_EXIST</a></code>            | No account exists at the <code>payee</code> address.                                                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code> | An account exists at <code>payee</code>, but it does not accept payments in <code>Currency</code>.                                                        |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_EACCOUNT_FROZEN">AccountFreezing::EACCOUNT_FROZEN</a></code>               | The <code>payee</code> account is frozen.                                                                                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_EMALFORMED_METADATA_SIGNATURE">DualAttestation::EMALFORMED_METADATA_SIGNATURE</a></code> | <code>metadata_signature</code> is not 64 bytes.                                                                                               |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_EINVALID_METADATA_SIGNATURE">DualAttestation::EINVALID_METADATA_SIGNATURE</a></code>   | <code>metadata_signature</code> does not verify on the against the <code>payee'</code>s <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code> <code>compliance_public_key</code> public key. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS">LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS</a></code>       | <code>payer</code> has exceeded its daily withdrawal limits for the backing coins of LBR.                                                      |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS">LibraAccount::EDEPOSIT_EXCEEDS_LIMITS</a></code>          | <code>payee</code> has exceeded its daily deposit limits for LBR.                                                                              |


<a name="@Related_Scripts_7"></a>

#### Related Scripts

* <code><a href="create_child_vasp_account.md#create_child_vasp_account">Script::create_child_vasp_account</a></code>
* <code><a href="create_parent_vasp_account.md#create_parent_vasp_account">Script::create_parent_vasp_account</a></code>
* <code><a href="add_currency_to_account.md#add_currency_to_account">Script::add_currency_to_account</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Currency&gt;(payer: &signer, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Currency&gt;(
    payer: &signer,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) {
    <b>let</b> payer_withdrawal_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(payer);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_pay_from">LibraAccount::pay_from</a>&lt;Currency&gt;(
        &payer_withdrawal_cap, payee, amount, metadata, metadata_signature
    );
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(payer_withdrawal_cap);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma verify;
</code></pre>


TODO(emmazzz): the following abort code checks don't work because there
are addition overflow aborts in AccountLimits not accompanied with abort
codes.


<a name="peer_to_peer_with_metadata_payer_addr$1"></a>


<pre><code><b>let</b> payer_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer);
<a name="peer_to_peer_with_metadata_cap$2"></a>
<b>let</b> cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">LibraAccount::spec_get_withdraw_cap</a>(payer_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">LibraAccount::ExtractWithdrawCapAbortsIf</a>{sender_addr: payer_addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_PayFromAbortsIf">LibraAccount::PayFromAbortsIf</a>&lt;Currency&gt;{cap: cap};
</code></pre>


The balances of payer and payee change by the correct amount.


<pre><code><b>ensures</b> payer_addr != payee
    ==&gt; <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payer_addr)
    == <b>old</b>(<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payer_addr)) - amount;
<b>ensures</b> payer_addr != payee
    ==&gt; <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payee)
    == <b>old</b>(<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payee)) + amount;
<b>ensures</b> payer_addr == payee
    ==&gt; <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payee)
    == <b>old</b>(<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;Currency&gt;(payee));
</code></pre>



</details>

<a name="mint_lbr"></a>

### Script `mint_lbr`



<a name="@Summary_8"></a>

#### Summary

Mints LBR from the sending account's constituent coins by depositing in the
on-chain LBR reserve. Deposits the newly-minted LBR into the sending
account. Can be sent by any account that can hold balances for the constituent
currencies for LBR and LBR.


<a name="@Technical_Description_9"></a>

#### Technical Description

Mints <code>amount_lbr</code> LBR from the sending account's constituent coins and deposits the
resulting LBR into the sending account.


<a name="@Events_10"></a>

##### Events

Successful execution of this script emits three events:
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> with the Coin1 currency code, and a
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> with the Coin2 currency code on <code>account</code>'s
<code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code> handle with the <code>amounts</code> for each event being the
components amounts of <code>amount_lbr</code> LBR; and
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> on <code>account</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code>
<code>received_events</code> handle with the LBR currency code and amount field equal to <code>amount_lbr</code>.


<a name="@Parameters_11"></a>

#### Parameters

| Name         | Type      | Description                                      |
| ------       | ------    | -------------                                    |
| <code>account</code>    | <code>&signer</code> | The signer reference of the sending account.     |
| <code>amount_lbr</code> | <code>u64</code>     | The amount of LBR (in microlibra) to be created. |


<a name="@Common_Abort_Conditions_12"></a>

#### Common Abort Conditions

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


<a name="@Related_Scripts_13"></a>

#### Related Scripts

* <code><a href="unmint_lbr.md#unmint_lbr">Script::unmint_lbr</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="overview.md#mint_lbr">mint_lbr</a>(account: &signer, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="overview.md#mint_lbr">mint_lbr</a>(account: &signer, amount_lbr: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_staple_lbr">LibraAccount::staple_lbr</a>(&withdraw_cap, amount_lbr);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma verify = <b>true</b>;
<a name="mint_lbr_account_addr$1"></a>
<b>let</b> account_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<a name="mint_lbr_cap$2"></a>
<b>let</b> cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">LibraAccount::spec_get_withdraw_cap</a>(account_addr);
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">LibraAccount::ExtractWithdrawCapAbortsIf</a>{sender_addr: account_addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBRAbortsIf">LibraAccount::StapleLBRAbortsIf</a>{cap: cap};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_StapleLBREnsures">LibraAccount::StapleLBREnsures</a>{cap: cap};
</code></pre>



</details>



<a name="@Index_14"></a>

## Index


-  [0x1::AccountFreezing](../../modules/doc/AccountFreezing.md#0x1_AccountFreezing)
-  [0x1::AccountLimits](../../modules/doc/AccountLimits.md#0x1_AccountLimits)
-  [0x1::Authenticator](../../modules/doc/Authenticator.md#0x1_Authenticator)
-  [0x1::ChainId](../../modules/doc/ChainId.md#0x1_ChainId)
-  [0x1::Coin1](../../modules/doc/Coin1.md#0x1_Coin1)
-  [0x1::Coin2](../../modules/doc/Coin2.md#0x1_Coin2)
-  [0x1::CoreAddresses](../../modules/doc/CoreAddresses.md#0x1_CoreAddresses)
-  [0x1::DesignatedDealer](../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer)
-  [0x1::DualAttestation](../../modules/doc/DualAttestation.md#0x1_DualAttestation)
-  [0x1::Errors](../../modules/doc/Errors.md#0x1_Errors)
-  [0x1::Event](../../modules/doc/Event.md#0x1_Event)
-  [0x1::FixedPoint32](../../modules/doc/FixedPoint32.md#0x1_FixedPoint32)
-  [0x1::Hash](../../modules/doc/Hash.md#0x1_Hash)
-  [0x1::LBR](../../modules/doc/LBR.md#0x1_LBR)
-  [0x1::LCS](../../modules/doc/LCS.md#0x1_LCS)
-  [0x1::Libra](../../modules/doc/Libra.md#0x1_Libra)
-  [0x1::LibraAccount](../../modules/doc/LibraAccount.md#0x1_LibraAccount)
-  [0x1::LibraConfig](../../modules/doc/LibraConfig.md#0x1_LibraConfig)
-  [0x1::LibraSystem](../../modules/doc/LibraSystem.md#0x1_LibraSystem)
-  [0x1::LibraTimestamp](../../modules/doc/LibraTimestamp.md#0x1_LibraTimestamp)
-  [0x1::LibraTransactionPublishingOption](../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption)
-  [0x1::LibraVersion](../../modules/doc/LibraVersion.md#0x1_LibraVersion)
-  [0x1::Option](../../modules/doc/Option.md#0x1_Option)
-  [0x1::RecoveryAddress](../../modules/doc/RecoveryAddress.md#0x1_RecoveryAddress)
-  [0x1::RegisteredCurrencies](../../modules/doc/RegisteredCurrencies.md#0x1_RegisteredCurrencies)
-  [0x1::Roles](../../modules/doc/Roles.md#0x1_Roles)
-  [0x1::SharedEd25519PublicKey](../../modules/doc/SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey)
-  [0x1::Signature](../../modules/doc/Signature.md#0x1_Signature)
-  [0x1::Signer](../../modules/doc/Signer.md#0x1_Signer)
-  [0x1::SlidingNonce](../../modules/doc/SlidingNonce.md#0x1_SlidingNonce)
-  [0x1::TransactionFee](../../modules/doc/TransactionFee.md#0x1_TransactionFee)
-  [0x1::VASP](../../modules/doc/VASP.md#0x1_VASP)
-  [0x1::ValidatorConfig](../../modules/doc/ValidatorConfig.md#0x1_ValidatorConfig)
-  [0x1::ValidatorOperatorConfig](../../modules/doc/ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig)
-  [0x1::Vector](../../modules/doc/Vector.md#0x1_Vector)
-  [add_currency_to_account](add_currency_to_account.md#add_currency_to_account)
-  [add_recovery_rotation_capability](add_recovery_rotation_capability.md#add_recovery_rotation_capability)
-  [add_to_script_allow_list](add_to_script_allow_list.md#add_to_script_allow_list)
-  [add_validator_and_reconfigure](add_validator_and_reconfigure.md#add_validator_and_reconfigure)
-  [burn](burn.md#burn)
-  [burn_txn_fees](burn_txn_fees.md#burn_txn_fees)
-  [cancel_burn](cancel_burn.md#cancel_burn)
-  [create_child_vasp_account](create_child_vasp_account.md#create_child_vasp_account)
-  [create_designated_dealer](create_designated_dealer.md#create_designated_dealer)
-  [create_parent_vasp_account](create_parent_vasp_account.md#create_parent_vasp_account)
-  [create_recovery_address](create_recovery_address.md#create_recovery_address)
-  [create_validator_account](create_validator_account.md#create_validator_account)
-  [create_validator_operator_account](create_validator_operator_account.md#create_validator_operator_account)
-  [freeze_account](freeze_account.md#freeze_account)
-  [mint_lbr](overview.md#mint_lbr)
-  [peer_to_peer_with_metadata](overview.md#peer_to_peer_with_metadata)
-  [preburn](preburn.md#preburn)
-  [publish_shared_ed25519_public_key](publish_shared_ed25519_public_key.md#publish_shared_ed25519_public_key)
-  [register_validator_config](register_validator_config.md#register_validator_config)
-  [remove_validator_and_reconfigure](remove_validator_and_reconfigure.md#remove_validator_and_reconfigure)
-  [rotate_authentication_key](rotate_authentication_key.md#rotate_authentication_key)
-  [rotate_authentication_key_with_nonce](rotate_authentication_key_with_nonce.md#rotate_authentication_key_with_nonce)
-  [rotate_authentication_key_with_nonce_admin](rotate_authentication_key_with_nonce_admin.md#rotate_authentication_key_with_nonce_admin)
-  [rotate_authentication_key_with_recovery_address](rotate_authentication_key_with_recovery_address.md#rotate_authentication_key_with_recovery_address)
-  [rotate_dual_attestation_info](rotate_dual_attestation_info.md#rotate_dual_attestation_info)
-  [rotate_shared_ed25519_public_key](rotate_shared_ed25519_public_key.md#rotate_shared_ed25519_public_key)
-  [set_validator_config_and_reconfigure](set_validator_config_and_reconfigure.md#set_validator_config_and_reconfigure)
-  [set_validator_operator](set_validator_operator.md#set_validator_operator)
-  [set_validator_operator_with_nonce_admin](set_validator_operator_with_nonce_admin.md#set_validator_operator_with_nonce_admin)
-  [tiered_mint](tiered_mint.md#tiered_mint)
-  [unfreeze_account](unfreeze_account.md#unfreeze_account)
-  [unmint_lbr](unmint_lbr.md#unmint_lbr)
-  [update_dual_attestation_limit](update_dual_attestation_limit.md#update_dual_attestation_limit)
-  [update_exchange_rate](update_exchange_rate.md#update_exchange_rate)
-  [update_libra_version](update_libra_version.md#update_libra_version)
-  [update_minting_ability](update_minting_ability.md#update_minting_ability)
