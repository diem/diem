
<a name="SCRIPT"></a>

# Script `update_dual_attestation_limit.move`

### Table of Contents

-  [Function `update_dual_attestation_limit`](#SCRIPT_update_dual_attestation_limit)



<a name="SCRIPT_update_dual_attestation_limit"></a>

## Function `update_dual_attestation_limit`

Update the dual attesation limit to
<code>new_micro_lbr_limit</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_update_dual_attestation_limit">update_dual_attestation_limit</a>(tc_account: &signer, sliding_nonce: u64, new_micro_lbr_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_update_dual_attestation_limit">update_dual_attestation_limit</a>(
    tc_account: &signer, sliding_nonce: u64, new_micro_lbr_limit: u64
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_set_microlibra_limit">DualAttestation::set_microlibra_limit</a>(tc_account, new_micro_lbr_limit);
}
</code></pre>



</details>
