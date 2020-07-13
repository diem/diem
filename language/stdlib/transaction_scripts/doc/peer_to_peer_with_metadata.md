
<a name="SCRIPT"></a>

# Script `peer_to_peer_with_metadata.move`

### Table of Contents

-  [Function `peer_to_peer_with_metadata`](#SCRIPT_peer_to_peer_with_metadata)
-  [Specification](#SCRIPT_Specification)
    -  [Function `peer_to_peer_with_metadata`](#SCRIPT_Specification_peer_to_peer_with_metadata)



<a name="SCRIPT_peer_to_peer_with_metadata"></a>

## Function `peer_to_peer_with_metadata`

Transfer
<code>amount</code> coins to
<code>recipient_address</code> with (optional)
associated metadata
<code>metadata</code> and (optional)
<code>signature</code> on the metadata, amount, and
sender address. The
<code>metadata</code> and
<code>signature</code> parameters are only required if
<code>amount</code> >= 1_000_000 micro LBR and the sender and recipient of the funds are two distinct VASPs.
Fails if there is no account at the recipient address or if the sender's balance is lower
than
<code>amount</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Token&gt;(payer: &signer, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Token&gt;(
    payer: &signer,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) {
    <b>let</b> payer_withdrawal_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(payer);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_pay_from">LibraAccount::pay_from</a>&lt;Token&gt;(&payer_withdrawal_cap, payee, amount, metadata, metadata_signature);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(payer_withdrawal_cap);
}
</code></pre>



</details>

<a name="SCRIPT_Specification"></a>

## Specification


<a name="SCRIPT_Specification_peer_to_peer_with_metadata"></a>

### Function `peer_to_peer_with_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;Token&gt;(payer: &signer, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



> TODO(emmazzz): This pre-condition is supposed to be a global property in LibraAccount:
The LibraAccount under addr holds either no withdraw capability
or the withdraw capability for addr itself.


<pre><code><b>requires</b> <a href="#SCRIPT_spec_get_withdraw_cap">spec_get_withdraw_cap</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer)).account_address
    == <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer);
<b>include</b> <a href="#SCRIPT_AbortsIfPayerInvalid">AbortsIfPayerInvalid</a>&lt;Token&gt;;
<b>include</b> <a href="#SCRIPT_AbortsIfPayeeInvalid">AbortsIfPayeeInvalid</a>&lt;Token&gt;;
<b>include</b> <a href="#SCRIPT_AbortsIfAmountInvalid">AbortsIfAmountInvalid</a>&lt;Token&gt;;
<b>include</b> <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_TravelRuleAppliesAbortsIf">DualAttestation::TravelRuleAppliesAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="#SCRIPT_AbortsIfAmountExceedsLimit">AbortsIfAmountExceedsLimit</a>&lt;Token&gt;;
</code></pre>


Post condition: the balances of payer and payee are changed correctly.


<pre><code><b>ensures</b> <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer) != payee
            ==&gt; <a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Token&gt;(payee) == <b>old</b>(<a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Token&gt;(payee)) + amount;
<b>ensures</b> <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer) != payee
            ==&gt; <a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Token&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer))
                == <b>old</b>(<a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Token&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer))) - amount;
</code></pre>


If payer and payee are the same, the balance does not change.


<pre><code><b>ensures</b> <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer) == payee
            ==&gt; <a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Token&gt;(payee) == <b>old</b>(<a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Token&gt;(payee));
</code></pre>




<pre><code>pragma verify = <b>true</b>, aborts_if_is_strict = <b>true</b>;
</code></pre>


Returns the
<code>withdrawal_capability</code> of LibraAccount under
<code>addr</code>.


<a name="SCRIPT_spec_get_withdraw_cap"></a>


<pre><code><b>define</b> <a href="#SCRIPT_spec_get_withdraw_cap">spec_get_withdraw_cap</a>(addr: address): WithdrawCapability {
    <a href="../../modules/doc/Option.md#0x1_Option_spec_value_inside">Option::spec_value_inside</a>(<b>global</b>&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr).withdrawal_capability)
}
</code></pre>


Returns the value of balance under addr.


<a name="SCRIPT_spec_balance_of"></a>


<pre><code><b>define</b> <a href="#SCRIPT_spec_balance_of">spec_balance_of</a>&lt;Token&gt;(addr: address): u64 {
    <b>global</b>&lt;Balance&lt;Token&gt;&gt;(addr).coin.value
}
</code></pre>




<a name="SCRIPT_AbortsIfPayerInvalid"></a>


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfPayerInvalid">AbortsIfPayerInvalid</a>&lt;Token&gt; {
    payer: signer;
    <b>aborts_if</b> !exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer));
    <b>aborts_if</b> <a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">AccountFreezing::spec_account_is_frozen</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer));
    <b>aborts_if</b> !exists&lt;Balance&lt;Token&gt;&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer));
}
</code></pre>


Aborts if payer's withdrawal_capability has been delegated.


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfPayerInvalid">AbortsIfPayerInvalid</a>&lt;Token&gt; {
    <b>aborts_if</b> <a href="../../modules/doc/Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(
        <b>global</b>&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(
            <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer)
        ).withdrawal_capability);
}
</code></pre>




<a name="SCRIPT_AbortsIfPayeeInvalid"></a>


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfPayeeInvalid">AbortsIfPayeeInvalid</a>&lt;Token&gt; {
    payee: address;
    <b>aborts_if</b> !exists&lt;<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payee);
    <b>aborts_if</b> <a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">AccountFreezing::spec_account_is_frozen</a>(payee);
    <b>aborts_if</b> !exists&lt;Balance&lt;Token&gt;&gt;(payee);
}
</code></pre>




<a name="SCRIPT_AbortsIfAmountInvalid"></a>


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountInvalid">AbortsIfAmountInvalid</a>&lt;Token&gt; {
    payer: &signer;
    payee: address;
    amount: u64;
    <b>aborts_if</b> amount == 0;
}
</code></pre>


Aborts if arithmetic overflow happens.


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountInvalid">AbortsIfAmountInvalid</a>&lt;Token&gt; {
    <b>aborts_if</b> <b>global</b>&lt;Balance&lt;Token&gt;&gt;(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer)).coin.value &lt; amount;
    <b>aborts_if</b> <a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer) != payee
            && <b>global</b>&lt;Balance&lt;Token&gt;&gt;(payee).coin.value + amount &gt; max_u64();
}
</code></pre>




<a name="SCRIPT_AbortsIfAmountExceedsLimit"></a>


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountExceedsLimit">AbortsIfAmountExceedsLimit</a>&lt;Token&gt; {
    payer: &signer;
    payee: address;
    amount: u64;
}
</code></pre>


Aborts if the amount exceeds payee's deposit limit.


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountExceedsLimit">AbortsIfAmountExceedsLimit</a>&lt;Token&gt; {
    <b>aborts_if</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">LibraAccount::spec_should_track_limits_for_account</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer), payee, <b>false</b>)
                && (!<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_has_account_operations_cap">LibraAccount::spec_has_account_operations_cap</a>()
                    || !<a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_spec_update_deposit_limits">AccountLimits::spec_update_deposit_limits</a>&lt;Token&gt;(
                            amount,
                            <a href="../../modules/doc/VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payee)
                        )
                    );
}
</code></pre>


Aborts if the amount exceeds payer's withdraw limit.


<pre><code><b>schema</b> <a href="#SCRIPT_AbortsIfAmountExceedsLimit">AbortsIfAmountExceedsLimit</a>&lt;Token&gt; {
    <b>aborts_if</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">LibraAccount::spec_should_track_limits_for_account</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer), payee, <b>true</b>)
                && (!<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_spec_has_account_operations_cap">LibraAccount::spec_has_account_operations_cap</a>()
                    || !<a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_spec_update_withdrawal_limits">AccountLimits::spec_update_withdrawal_limits</a>&lt;Token&gt;(
                            amount,
                            <a href="../../modules/doc/VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(<a href="../../modules/doc/Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(payer))
                        )
                    );
}
</code></pre>



> TODO(emmazzz): turn verify on when non-termination issue is resolved.


<pre><code>pragma verify = <b>false</b>;
</code></pre>
