
<a name="SCRIPT"></a>

# Script `update_travel_rule_limit.move`

### Table of Contents

-  [Function `update_travel_rule_limit`](#SCRIPT_update_travel_rule_limit)



<a name="SCRIPT_update_travel_rule_limit"></a>

## Function `update_travel_rule_limit`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_update_travel_rule_limit">update_travel_rule_limit</a>(tc_account: &signer, sliding_nonce: u64, new_micro_lbr_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_update_travel_rule_limit">update_travel_rule_limit</a>(tc_account: &signer, sliding_nonce: u64, new_micro_lbr_limit: u64) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <b>let</b> cap = <a href="../../modules/doc/Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>&lt;UpdateDualAttestationThreshold&gt;(tc_account);
    <a href="../../modules/doc/DualAttestationLimit.md#0x1_DualAttestationLimit_set_microlibra_limit">DualAttestationLimit::set_microlibra_limit</a>(&cap, <a href="../../modules/doc/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(tc_account), new_micro_lbr_limit);
    <a href="../../modules/doc/Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(tc_account, cap);
}
</code></pre>



</details>
