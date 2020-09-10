
<a name="SCRIPT"></a>

# Script `peer_to_peer_with_metadata.move`

### Table of Contents

-  [Function `peer_to_peer_with_metadata`](#SCRIPT_peer_to_peer_with_metadata)
        -  [Events](#SCRIPT_@Events)
        -  [Common Aborts](#SCRIPT_@Common_Aborts)
        -  [Dual Attestation Aborts](#SCRIPT_@Dual_Attestation_Aborts)
        -  [Other Aborts](#SCRIPT_@Other_Aborts)
-  [Specification](#SCRIPT_Specification)
    -  [Function `peer_to_peer_with_metadata`](#SCRIPT_Specification_peer_to_peer_with_metadata)
        -  [Post conditions](#SCRIPT_@Post_conditions)
        -  [Abort conditions](#SCRIPT_@Abort_conditions)



<a name="SCRIPT_peer_to_peer_with_metadata"></a>

## Function `peer_to_peer_with_metadata`

Transfer
<code>amount</code> coins of type
<code>Currency</code> from
<code>payer</code> to
<code>payee</code> with (optional) associated
<code>metadata</code> and an (optional)
<code>metadata_signature</code> on the message
<code>metadata</code> |
<code><a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(payer)</code> |
<code>amount</code> |
<code>DualAttestation::DOMAIN_SEPARATOR</code>.
The
<code>metadata</code> and
<code>metadata_signature</code> parameters are only required if
<code>amount</code> >=
<code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_get_cur_microlibra_limit">DualAttestation::get_cur_microlibra_limit</a></code> LBR and
<code>payer</code> and
<code>payee</code> are distinct VASPs.
However, a transaction sender can opt in to dual attestation even when it is not required (e.g., a DesignatedDealer -> VASP payment) by providing a non-empty
<code>metadata_signature</code>.
Standardized
<code>metadata</code> LCS format can be found in
<code>libra_types::transaction::metadata::Metadata</code>.


<a name="SCRIPT_@Events"></a>

#### Events

When this script executes without aborting, it emits two events:
<code>SentPaymentEvent { amount, currency_code = Currency, payee, metadata }</code>
on
<code>payer</code>'s
<code>LibraAccount::sent_events</code> handle, and
<code>ReceivedPaymentEvent { amount, currency_code = Currency, payer, metadata }</code>
on
<code>payee</code>'s
<code>LibraAccount::received_events</code> handle.


<a name="SCRIPT_@Common_Aborts"></a>

#### Common Aborts

These aborts can in occur in any payment.
* Aborts with
<code>LibraAccount::EINSUFFICIENT_BALANCE</code> if
<code>amount</code> is greater than
<code>payer</code>'s balance in
<code>Currency</code>.
* Aborts with
<code>LibraAccount::ECOIN_DEPOSIT_IS_ZERO</code> if
<code>amount</code> is zero.
* Aborts with
<code>LibraAccount::EPAYEE_DOES_NOT_EXIST</code> if no account exists at the address
<code>payee</code>.
* Aborts with
<code>LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</code> if an account exists at
<code>payee</code>, but it does not accept payments in
<code>Currency</code>.


<a name="SCRIPT_@Dual_Attestation_Aborts"></a>

#### Dual Attestation Aborts

These aborts can occur in any payment subject to dual attestation.
* Aborts with
<code>DualAttestation::EMALFORMED_METADATA_SIGNATURE</code> if
<code>metadata_signature</code>'s is not 64 bytes.
* Aborts with
<code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation">DualAttestation</a>:EINVALID_METADATA_SIGNATURE</code> if
<code>metadata_signature</code> does not verify on the message
<code>metadata</code> |
<code>payer</code> |
<code>value</code> |
<code>DOMAIN_SEPARATOR</code> using the
<code>compliance_public_key</code> published in the
<code>payee</code>'s
<code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Credential">DualAttestation::Credential</a></code> resource.


<a name="SCRIPT_@Other_Aborts"></a>

#### Other Aborts

These aborts should only happen when
<code>payer</code> or
<code>payee</code> have account limit restrictions or
have been frozen by Libra administrators.
* Aborts with
<code>LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS</code> if
<code>payer</code> has exceeded their daily
withdrawal limits.
* Aborts with
<code>LibraAccount::EDEPOSIT_EXCEEDS_LIMITS</code> if
<code>payee</code> has exceeded their daily deposit limits.
* Aborts with
<code>LibraAccount::EACCOUNT_FROZEN</code> if
<code>payer</code>'s account is frozen.


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
