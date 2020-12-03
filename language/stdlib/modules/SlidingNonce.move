address 0x1 {

/// Allows transactions to be executed out-of-order while ensuring that they are executed at most once.
/// Nonces are assigned to transactions off-chain by clients submitting the transactions.
/// It maintains a sliding window bitvector of 128 flags.  A flag of 0 indicates that the transaction
/// with that nonce has not yet been executed.
/// When nonce X is recorded, all transactions with nonces lower then X-128 will abort.
module SlidingNonce {
    use 0x1::Signer;
    use 0x1::Errors;

    resource struct SlidingNonce {
        /// Minimum nonce in sliding window. All transactions with smaller
        /// nonces will be automatically rejected, since the window cannot
        /// tell whether they have been executed or not.
        min_nonce: u64,
        /// Bit-vector of window of nonce values
        nonce_mask: u128,
    }

    /// The `SlidingNonce` resource is in an invalid state
    const ESLIDING_NONCE: u64 = 0;
    /// The nonce aborted because it's too old (nonce smaller than `min_nonce`)
    const ENONCE_TOO_OLD: u64 = 1;
    /// The nonce is too large - this protects against nonce exhaustion
    const ENONCE_TOO_NEW: u64 = 2;
    /// The nonce was already recorded previously
    const ENONCE_ALREADY_RECORDED: u64 = 3;
    /// The sliding nonce resource was already published
    const ENONCE_ALREADY_PUBLISHED: u64 = 4;

    /// Size of SlidingNonce::nonce_mask in bits.
    const NONCE_MASK_SIZE: u64 = 128;

    /// Calls `try_record_nonce` and aborts transaction if returned code is non-0
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
        /// >Note: Verification is turned off. For verifying callers, this is effectively abstracted into a function
        /// that returns arbitrary results because `spec_try_record_nonce` is uninterpreted.
        pragma opaque, verify = false;
        ensures result == spec_try_record_nonce(account, seq_nonce);
        aborts_if !exists<SlidingNonce>(Signer::spec_address_of(account)) with Errors::NOT_PUBLISHED;
    }

    /// Specification version of `Self::try_record_nonce`.
    spec define spec_try_record_nonce(account: signer, seq_nonce: u64): u64;

    /// Publishes nonce resource for `account`
    /// This is required before other functions in this module can be called for `account
    public fun publish(account: &signer) {
        assert(!exists<SlidingNonce>(Signer::address_of(account)), Errors::already_published(ENONCE_ALREADY_PUBLISHED));
        move_to(account, SlidingNonce {  min_nonce: 0, nonce_mask: 0 });
    }
    spec fun publish {
        pragma opaque;
        modifies global<SlidingNonce>(Signer::spec_address_of(account));
        aborts_if exists<SlidingNonce>(Signer::spec_address_of(account)) with Errors::ALREADY_PUBLISHED;
        ensures exists<SlidingNonce>(Signer::spec_address_of(account));
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    spec module {
        use 0x1::CoreAddresses;
        use 0x1::DiemTimestamp;

        /// Sliding nonces are initialized at Diem root and treasury compliance addresses
        invariant [global] DiemTimestamp::is_operating()
            ==> exists<SlidingNonce>(CoreAddresses::DIEM_ROOT_ADDRESS());

        invariant [global] DiemTimestamp::is_operating()
            ==> exists<SlidingNonce>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS());

        // In the current code, only Diem root and Treasury compliance have sliding nonces.
        // That is a difficult cross-module invariant to prove (it depends on Genesis and
        // DiemAccount). Additional modules could be added that call the publish functions
        // in this module to publish sliding nonces on other accounts.  Anyway, this property
        // is probably not very important.
    }
}
}
