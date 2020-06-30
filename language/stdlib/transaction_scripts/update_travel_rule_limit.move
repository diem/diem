script {
    use 0x1::Signer;
    use 0x1::SlidingNonce;
    use 0x1::DualAttestationLimit::{Self, UpdateDualAttestationThreshold};
    use 0x1::Roles;

    /// Update the travel rule limit to `new_micro_lbr_limit`.
    fun update_travel_rule_limit(tc_account: &signer, sliding_nonce: u64, new_micro_lbr_limit: u64) {
        SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
        let cap = Roles::extract_privilege_to_capability<UpdateDualAttestationThreshold>(tc_account);
        DualAttestationLimit::set_microlibra_limit(&cap, Signer::address_of(tc_account), new_micro_lbr_limit);
        Roles::restore_capability_to_privilege(tc_account, cap);
    }
}
