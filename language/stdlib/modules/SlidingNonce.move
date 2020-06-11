address 0x1 {
module SlidingNonce {
    use 0x1::Association;
    use 0x1::Signer;

    // This struct keep last 128 nonce values in a bit map nonce_mask
    // We assume that nonce are generated incrementally, but certain permutation is allowed when nonce are recorded
    // For example you can record nonce 10 and then record nonce 9
    // When nonce X is recorded, all nonce lower then X-128 will be rejected with code 10001(see below)
    // In a nutshell, min_nonce records minimal nonce allowed
    // And nonce_mask contains a bitmap for nonce in range [min_nonce; min_nonce+127]
    resource struct SlidingNonce {
        min_nonce: u64,
        nonce_mask: u128,
    }

    // Calls try_record_nonce and aborts transaction if returned code is non-0
    public fun record_nonce_or_abort(account: &signer, seq_nonce: u64) acquires SlidingNonce {
        let code = try_record_nonce(account, seq_nonce);
        assert(code == 0, code);
    }

    // Tries to record this nonce in the account.
    // Returns 0 if a nonce was recorded and non-0 otherwise
    // Reasons for nonce to be rejected:
    // * code 10001: This nonce is too old and impossible to ensure whether it's duplicated or not
    // * code 10002: This nonce is too far in the future - this is not allowed to protect against nonce exhaustion
    // * code 10003: This nonce was already recorded previously
    public fun try_record_nonce(account: &signer, seq_nonce: u64): u64 acquires SlidingNonce {
        if (seq_nonce == 0) {
            return 0
        };
        let t = borrow_global_mut<SlidingNonce>(Signer::address_of(account));
        if (t.min_nonce > seq_nonce) {
            return 10001
        };
        let jump_limit = 10000; // Don't allow giant leaps in nonce to protect against nonce exhaustion
        if (t.min_nonce + jump_limit <= seq_nonce) {
            return 10002
        };
        let bit_pos = seq_nonce - t.min_nonce;
        let nonce_mask_size = 128; // size of SlidingNonce::nonce_mask in bits. no constants in move?
        if (bit_pos >= nonce_mask_size) {
            let shift = (bit_pos - nonce_mask_size + 1);
            if(shift >= nonce_mask_size) {
                t.nonce_mask = 0;
                t.min_nonce = seq_nonce + 1 - nonce_mask_size;
            } else {
                t.nonce_mask = t.nonce_mask >> (shift as u8);
                t.min_nonce = t.min_nonce + shift;
            }
        };
        let bit_pos = seq_nonce - t.min_nonce;
        let set = 1u128 << (bit_pos as u8);
        if (t.nonce_mask & set != 0) {
            return 10003
        };
        t.nonce_mask = t.nonce_mask | set;
        0
    }

    // Publishes nonce resource for `account`
    // This is required before other functions in this module can be called for `account
    public fun publish(account: &signer) {
        move_to(account, SlidingNonce {  min_nonce: 0, nonce_mask: 0 });
    }

    // Publishes nonce resource into specific account
    // Only association can create this resource for different account
    // Alternative is publish_nonce_resource_for_user that publishes resource into current account
    public fun publish_nonce_resource(association: &signer, account: &signer) {
        Association::assert_is_root(association);
        let new_resource = SlidingNonce {
            min_nonce: 0,
            nonce_mask: 0,
        };
        move_to(account, new_resource)
    }
}
}
