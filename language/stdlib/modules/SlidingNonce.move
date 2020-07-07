address 0x1 {
module SlidingNonce {
    use 0x1::Signer;
    use 0x1::Roles;

    /// This struct keep last 128 nonce values in a bit map nonce_mask
    /// We assume that nonce are generated incrementally, but certain permutation is allowed when nonce are recorded
    /// For example you can record nonce 10 and then record nonce 9
    /// When nonce X is recorded, all nonce lower then X-128 will be rejected with code 10001(see below)
    /// In a nutshell, min_nonce records minimal nonce allowed
    /// And nonce_mask contains a bitmap for nonce in range [min_nonce; min_nonce+127]
    resource struct SlidingNonce {
        min_nonce: u64,
        nonce_mask: u128,
    }

    /// The nonce is too old and impossible to ensure whether it's duplicated or not
    const ENONCE_TOO_OLD: u64 = 1;
    /// The nonce is too far in the future - this is not allowed to protect against nonce exhaustion
    const ENONCE_TOO_NEW: u64 = 2;
    /// The nonce was already recorded previously
    const ENONCE_ALREADY_RECORDED: u64 = 3;
    /// Calling account doesn't have sufficient privileges to create a sliding nonce resource
    const ENOT_LIBRA_ROOT: u64 = 4;

    /// Size of SlidingNonce::nonce_mask in bits.
    const NONCE_MASK_SIZE: u64 = 128;

    /// Calls try_record_nonce and aborts transaction if returned code is non-0
    public fun record_nonce_or_abort(account: &signer, seq_nonce: u64) acquires SlidingNonce {
        let code = try_record_nonce(account, seq_nonce);
        assert(code == 0, code);
    }

    /// Tries to record this nonce in the account.
    /// Returns 0 if a nonce was recorded and non-0 otherwise
    public fun try_record_nonce(account: &signer, seq_nonce: u64): u64 acquires SlidingNonce {
        if (seq_nonce == 0) {
            return 0
        };
        let t = borrow_global_mut<SlidingNonce>(Signer::address_of(account));
        if (t.min_nonce > seq_nonce) {
            return ENONCE_TOO_OLD
        };
        let jump_limit = 10000; // Don't allow giant leaps in nonce to protect against nonce exhaustion
        if (t.min_nonce + jump_limit <= seq_nonce) {
            return ENONCE_TOO_NEW
        };
        let bit_pos = seq_nonce - t.min_nonce;
        if (bit_pos >= NONCE_MASK_SIZE) {
            let shift = (bit_pos - NONCE_MASK_SIZE + 1);
            if(shift >= NONCE_MASK_SIZE) {
                t.nonce_mask = 0;
                t.min_nonce = seq_nonce + 1 - NONCE_MASK_SIZE;
            } else {
                t.nonce_mask = t.nonce_mask >> (shift as u8);
                t.min_nonce = t.min_nonce + shift;
            }
        };
        let bit_pos = seq_nonce - t.min_nonce;
        let set = 1u128 << (bit_pos as u8);
        if (t.nonce_mask & set != 0) {
            return ENONCE_ALREADY_RECORDED
        };
        t.nonce_mask = t.nonce_mask | set;
        0
    }

    /// Publishes nonce resource for `account`
    /// This is required before other functions in this module can be called for `account
    public fun publish(account: &signer) {
        move_to(account, SlidingNonce {  min_nonce: 0, nonce_mask: 0 });
    }

    /// Publishes nonce resource into specific account
    /// Only the libra root account can create this resource for different accounts
    public fun publish_nonce_resource(
        lr_account: &signer,
        account: &signer
    ) {
        assert(Roles::has_libra_root_role(lr_account), ENOT_LIBRA_ROOT);
        let new_resource = SlidingNonce {
            min_nonce: 0,
            nonce_mask: 0,
        };
        move_to(account, new_resource)
    }
}
}
