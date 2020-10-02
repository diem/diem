address 0x1 {
module SlidingNonce {
    use 0x1::Signer;
    use 0x1::Roles;
    use 0x1::Errors;

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

    /// The `SlidingNonce` resource is in an invalid state
    const ESLIDING_NONCE: u64 = 0;
    /// The nonce is too old and impossible to ensure whether it's duplicated or not
    const ENONCE_TOO_OLD: u64 = 1;
    /// The nonce is too far in the future - this is not allowed to protect against nonce exhaustion
    const ENONCE_TOO_NEW: u64 = 2;
    /// The nonce was already recorded previously
    const ENONCE_ALREADY_RECORDED: u64 = 3;
    /// The sliding nonce resource was already published
    const ENONCE_ALREADY_PUBLISHED: u64 = 4;

    /// Size of SlidingNonce::nonce_mask in bits.
    const NONCE_MASK_SIZE: u64 = 128;

    /// Calls try_record_nonce and aborts transaction if returned code is non-0
    public fun record_nonce_or_abort(account: &signer, seq_nonce: u64) acquires SlidingNonce {
        let code = try_record_nonce(account, seq_nonce);
        assert(code == 0, Errors::invalid_argument(code));
    }

    spec fun record_nonce_or_abort {
        include RecordNonceAbortsIf;
    }

    spec schema RecordNonceAbortsIf {
        account: signer;
        seq_nonce: u64;
        aborts_if !exists<SlidingNonce>(Signer::spec_address_of(account)) with Errors::NOT_PUBLISHED;
        aborts_if spec_try_record_nonce(account, seq_nonce) != 0 with Errors::INVALID_ARGUMENT;
    }

    /// Tries to record this nonce in the account.
    /// Returns 0 if a nonce was recorded and non-0 otherwise
    public fun try_record_nonce(account: &signer, seq_nonce: u64): u64 acquires SlidingNonce {
        if (seq_nonce == 0) {
            return 0
        };
        assert(exists<SlidingNonce>(Signer::address_of(account)), Errors::not_published(ESLIDING_NONCE));
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

    spec fun try_record_nonce {
        /// It is currently assumed that this function raises no arithmetic overflow/underflow.
        pragma opaque, verify = false;
        ensures result == spec_try_record_nonce(account, seq_nonce);
        aborts_if !exists<SlidingNonce>(Signer::spec_address_of(account)) with Errors::NOT_PUBLISHED;
    }

    /// Specification version of `Self::try_record_nonce`.
    spec define spec_try_record_nonce(account: signer, seq_nonce: u64): u64;

    /// Publishes nonce resource for `account`
    /// This is required before other functions in this module can be called for `account
    public fun publish(account: &signer) {
        assert(!exists<SlidingNonce>(Signer::address_of(account)), Errors::invalid_argument(ENONCE_ALREADY_PUBLISHED));
        move_to(account, SlidingNonce {  min_nonce: 0, nonce_mask: 0 });
    }

    /// Publishes nonce resource into specific account
    /// Only the libra root account can create this resource for different accounts
    public fun publish_nonce_resource(
        lr_account: &signer,
        account: &signer
    ) {
        Roles::assert_libra_root(lr_account);
        let new_resource = SlidingNonce {
            min_nonce: 0,
            nonce_mask: 0,
        };
        assert(!exists<SlidingNonce>(Signer::address_of(account)),
                Errors::invalid_argument(ENONCE_ALREADY_PUBLISHED));
        move_to(account, new_resource);
    }

}
}
