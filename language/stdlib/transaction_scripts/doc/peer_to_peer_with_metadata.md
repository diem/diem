
<a name="SCRIPT"></a>

# Script `peer_to_peer_with_metadata.move`

### Table of Contents

-  [Function `peer_to_peer_with_metadata`](#SCRIPT_peer_to_peer_with_metadata)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
        -  [Events](#SCRIPT_@Events)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)
-  [Specification](#SCRIPT_Specification)
    -  [Function `peer_to_peer_with_metadata`](#SCRIPT_Specification_peer_to_peer_with_metadata)
        -  [Post conditions](#SCRIPT_@Post_conditions)
        -  [Abort conditions](#SCRIPT_@Abort_conditions)



<a name="SCRIPT_peer_to_peer_with_metadata"></a>

## Function `peer_to_peer_with_metadata`


<a name="SCRIPT_@Summary"></a>

### Summary

Transfers a given number of coins in a specified currency from one account to another.
Transfers over a specified amount defined on-chain that are between two different VASPs, or
other accounts that have opted-in will be subject to on-chain checks to ensure the receiver has
agreed to receive the coins.  This transaction can be sent by any account that can hold a
balance, and to any account that can hold a balance. Both accounts must hold balances in the
currency being transacted.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description


Transfers <code>amount</code> coins of type <code>Currency</code> from <code>payer</code> to <code>payee</code> with (optional) associated
<code>metadata</code> and an (optional) <code>metadata_signature</code> on the message
<code>metadata</code> | <code><a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(payer)</code> | <code>amount</code> | <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_DOMAIN_SEPARATOR">DualAttestation::DOMAIN_SEPARATOR</a></code>.
The <code>metadata</code> and <code>metadata_signature</code> parameters are only required if <code>amount</code> >=
<code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_get_cur_microlibra_limit">DualAttestation::get_cur_microlibra_limit</a></code> LBR and <code>payer</code> and <code>payee</code> are distinct VASPs.
However, a transaction sender can opt in to dual attestation even when it is not required
(e.g., a DesignatedDealer -> VASP payment) by providing a non-empty <code>metadata_signature</code>.
Standardized <code>metadata</code> LCS format can be found in <code>libra_types::transaction::metadata::Metadata</code>.


<a name="SCRIPT_@Events"></a>

#### Events

Successful execution of this script emits two events:
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a></code> on <code>payer</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>sent_events</code> handle; and
* A <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a></code> on <code>payee</code>'s <code><a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a></code> <code>received_events</code> handle.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name                 | Type         | Description                                                                                                                  |
| ------               | ------       | -------------                                                                                                                |
| <code>Currency</code>           | Type         | The Move type for the <code>Currency</code> being sent in this transaction. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>payer</code>              | <code>&signer</code>    | The signer reference of the sending account that coins are being transferred from.                                           |
| <code>payee</code>              | <code>address</code>    | The address of the account the coins are being transferred to.                                                               |
| <code>metadata</code>           | <code>vector&lt;u8&gt;</code> | Optional metadata about this payment.                                                                                        |
| <code>metadata_signature</code> | <code>vector&lt;u8&gt;</code> | Optional signature over <code>metadata</code> and payment information. See                                                              |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

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


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Script::create_child_vasp_account</code>
* <code>Script::create_parent_vasp_account</code>
* <code>Script::add_currency_to_account</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Currency&gt;(payer: &signer, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Currency&gt;(
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

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_peer_to_peer_with_metadata"></a>

### Function `peer_to_peer_with_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Currency&gt;(payer: &signer, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<a name="SCRIPT_payer_addr$2"></a>
<b>let</b> payer_addr = <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer);
</code></pre>



<a name="SCRIPT_@Post_conditions"></a>

#### Post conditions

The balances of payer and payee are changed correctly if payer and payee are different.


<pre><code><b>ensures</b> payer_addr != payee
            ==&gt; <a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Currency&gt;(payee) == <b>old</b>(<a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Currency&gt;(payee)) + amount;
<b>ensures</b> payer_addr != payee
            ==&gt; <a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Currency&gt;(payer_addr) == <b>old</b>(<a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Currency&gt;(payer_addr)) - amount;
</code></pre>


If payer and payee are the same, the balance does not change.


<pre><code><b>ensures</b> payer_addr == payee ==&gt; <a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Currency&gt;(payee) == <b>old</b>(<a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Currency&gt;(payee));
</code></pre>



<a name="SCRIPT_@Abort_conditions"></a>

#### Abort conditions



<pre><code><b>include</b> <a href="#SCRIPT_AbortsIfPayerInvalid">AbortsIfPayerInvalid</a>&lt;Currency&gt;{payer: payer_addr};
<b>include</b> <a href="#SCRIPT_AbortsIfPayeeInvalid">AbortsIfPayeeInvalid</a>&lt;Currency&gt;;
<b>include</b> <a href="#SCRIPT_AbortsIfAmountInvalid">AbortsIfAmountInvalid</a>&lt;Currency&gt;{payer: payer_addr};
<b>include</b> <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_AssertPaymentOkAbortsIf">DualAttestation::AssertPaymentOkAbortsIf</a>&lt;Currency&gt;{payer: payer_addr, value: amount};
<b>include</b> <a href="#SCRIPT_AbortsIfAmountExceedsLimit">AbortsIfAmountExceedsLimit</a>&lt;Currency&gt;{payer: payer_addr};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">LibraAccount::spec_should_track_limits_for_account</a>&lt;Currency&gt;(payer_addr, payee, <b>false</b>) ==&gt;
            <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_UpdateDepositLimitsAbortsIf">AccountLimits::UpdateDepositLimitsAbortsIf</a>&lt;Currency&gt; {
                addr: <a href="../../modules/doc/VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payee),
            };
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">LibraAccount::spec_should_track_limits_for_account</a>&lt;Currency&gt;(payer_addr, payee, <b>true</b>) ==&gt;
            <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_UpdateWithdrawalLimitsAbortsIf">AccountLimits::UpdateWithdrawalLimitsAbortsIf</a>&lt;Currency&gt; {
                addr: <a href="../../modules/doc/VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payer_addr),
            };
</code></pre>




<pre><code>pragma verify = <b>true</b>, aborts_if_is_strict = <b>true</b>;
</code></pre>


Returns the value of balance under addr.


<a name="SCRIPT_spec_balance_of"></a>


<pre><code><b>define</b> <a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Currency&gt;(addr: address): u64 {
    <b>global</b>&lt;Balance&lt;Currency&gt;&gt;(addr).coin.value
}
</code></pre>




<a name="SCRIPT_AbortsIfPayerInvalid"></a>


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfPayerInvalid">AbortsIfPayerInvalid</a>&lt;Currency&gt; {
    payer: address;
    <b>aborts_if</b> !exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer);
    <b>aborts_if</b> <a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">AccountFreezing::account_is_frozen</a>(payer);
    <b>aborts_if</b> !exists&lt;Balance&lt;Currency&gt;&gt;(payer);
}
</code></pre>


Aborts if payer's withdrawal_capability has been delegated.


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfPayerInvalid">AbortsIfPayerInvalid</a>&lt;Currency&gt; {
    <b>aborts_if</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_delegated_withdraw_capability">LibraAccount::delegated_withdraw_capability</a>(payer);
}
</code></pre>




<a name="SCRIPT_AbortsIfPayeeInvalid"></a>


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfPayeeInvalid">AbortsIfPayeeInvalid</a>&lt;Currency&gt; {
    payee: address;
    <b>aborts_if</b> !exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payee);
    <b>aborts_if</b> <a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">AccountFreezing::account_is_frozen</a>(payee);
    <b>aborts_if</b> !exists&lt;Balance&lt;Currency&gt;&gt;(payee);
}
</code></pre>




<a name="SCRIPT_AbortsIfAmountInvalid"></a>


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountInvalid">AbortsIfAmountInvalid</a>&lt;Currency&gt; {
    payer: address;
    payee: address;
    amount: u64;
    <b>aborts_if</b> amount == 0;
}
</code></pre>


Aborts if arithmetic overflow happens.


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountInvalid">AbortsIfAmountInvalid</a>&lt;Currency&gt; {
    <b>aborts_if</b> <b>global</b>&lt;Balance&lt;Currency&gt;&gt;(payer).coin.value &lt; amount;
    <b>aborts_if</b> payer != payee
            && <b>global</b>&lt;Balance&lt;Currency&gt;&gt;(payee).coin.value + amount &gt; max_u64();
}
</code></pre>




<a name="SCRIPT_AbortsIfAmountExceedsLimit"></a>


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountExceedsLimit">AbortsIfAmountExceedsLimit</a>&lt;Currency&gt; {
    payer: address;
    payee: address;
    amount: u64;
}
</code></pre>


Aborts if the amount exceeds payee's deposit limit.


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountExceedsLimit">AbortsIfAmountExceedsLimit</a>&lt;Currency&gt; {
    <b>aborts_if</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">LibraAccount::spec_should_track_limits_for_account</a>&lt;Currency&gt;(payer, payee, <b>false</b>)
                && (!<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_has_account_operations_cap">LibraAccount::spec_has_account_operations_cap</a>()
                    || !<a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_spec_update_deposit_limits">AccountLimits::spec_update_deposit_limits</a>&lt;Currency&gt;(
                            amount,
                            <a href="../../modules/doc/VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payee)
                        )
                    );
}
</code></pre>


Aborts if the amount exceeds payer's withdraw limit.


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountExceedsLimit">AbortsIfAmountExceedsLimit</a>&lt;Currency&gt; {
    <b>aborts_if</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">LibraAccount::spec_should_track_limits_for_account</a>&lt;Currency&gt;(payer, payee, <b>true</b>)
                && (!<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_has_account_operations_cap">LibraAccount::spec_has_account_operations_cap</a>()
                    || !<a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_spec_update_withdrawal_limits">AccountLimits::spec_update_withdrawal_limits</a>&lt;Currency&gt;(
                            amount,
                            <a href="../../modules/doc/VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payer)
                        )
                    );
}
</code></pre>




<pre><code>pragma verify = <b>true</b>;
</code></pre>
