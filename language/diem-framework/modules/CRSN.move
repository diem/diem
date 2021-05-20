address 0x1 {
/// A module implementing conflict-resistant sequence numbers (CRSNs).
/// The specification, and formal description of the acceptance and rejection
/// criteria, force expiration and window shifting of CRSNs are described in DIP-168.
module CRSN {
    use 0x1::BitVector::{Self, BitVector};
    use 0x1::Signer;
    use 0x1::Errors;

    friend 0x1::DiemAccount;

    /// A CRSN  represents a finite slice or window of an "infinite" bitvector
    /// starting at zero with a size `k` defined dynamically at the time of
    /// publication of CRSN resource. The `min_nonce` defines the left-hand
    /// side of the slice, and the slice's state is held in `slots` and is of size `k`.
    /// Diagrammatically:
    /// ```
    /// 1111...000000100001000000...0100001000000...0000...
    ///        ^             ...                ^
    ///        |____..._____slots______...______|
    ///     min_nonce                       min_nonce + k - 1
    /// ```
    struct CRSN has key {
        min_nonce: u64,
        size: u64,
        slots: BitVector,
    }

    /// No CRSN resource exists
    const ENO_CRSN: u64 = 0;
    /// A CRSN resource wasn't expected, but one was found
    const EHAS_CRSN: u64 = 1;
    /// The size given to the CRSN at the time of publishing was zero, which is not supported
    const EZERO_SIZE_CRSN: u64 = 2;


    /// Publish a DSN under `account`. Cannot already have a DSN published.
    public(friend) fun publish(account: &signer, min_nonce: u64, size: u64) {
        assert(!has_crsn(Signer::address_of(account)), Errors::invalid_state(EHAS_CRSN));
        assert(size > 0, Errors::invalid_argument(EZERO_SIZE_CRSN));
        move_to(account, CRSN {
            min_nonce,
            size,
            slots: BitVector::new(size),
        })
    }

    /// Record `sequence_nonce` under the `account`. Returns true if
    /// `sequence_nonce` is accepted, returns false if the `sequence_nonce` is rejected.
    public(friend) fun record(account: &signer, sequence_nonce: u64): bool
    acquires CRSN {
        if (check(account, sequence_nonce)) {
            // CRSN exists by `check`.
            let crsn = borrow_global_mut<CRSN>(Signer::address_of(account));
            // accept nonce
            let scaled_nonce = sequence_nonce - crsn.min_nonce;
            BitVector::set(&mut crsn.slots, scaled_nonce);
            shift_window_right(crsn);
            true
        } else {
            false
        }
    }

    /// A stateless version of `record`: returns `true` if the `sequence_nonce`
    /// will be accepted, and `false` otherwise.
    public(friend) fun check(account: &signer, sequence_nonce: u64): bool
    acquires CRSN {
        let addr = Signer::address_of(account);
        assert(has_crsn(addr), Errors::invalid_state(ENO_CRSN));
        let crsn = borrow_global_mut<CRSN>(addr);

        // Don't accept if it's outside of the window
        if ((sequence_nonce < crsn.min_nonce) ||
            (sequence_nonce >= crsn.min_nonce + BitVector::length(&crsn.slots))) {
            return false
        };

        // scaled nonce is the index in the window
        let scaled_nonce = sequence_nonce - crsn.min_nonce;

        // Bit already set, reject
        if (BitVector::is_index_set(&crsn.slots, scaled_nonce)) return false;

        // otherwise, accept
        true

    }

    /// Force expire transactions by forcibly shifting the window by
    /// `shift_amount`. After the window has been shifted by `shift_amount` it is
    /// then shifted over set bits as define by the `shift_window_right` function.
    public fun force_expire(account: &signer, shift_amount: u64)
    acquires CRSN {
        if (shift_amount == 0) return;
        let addr = Signer::address_of(account);
        assert(has_crsn(addr), Errors::invalid_state(ENO_CRSN));
        let crsn = borrow_global_mut<CRSN>(addr);

        BitVector::shift_left(&mut crsn.slots, shift_amount);

        crsn.min_nonce = crsn.min_nonce + shift_amount;
        // shift over any set bits
        shift_window_right(crsn);
    }

    /// Return whether this address has a CRSN resource published under it.
    public fun has_crsn(addr: address): bool {
        exists<CRSN>(addr)
    }

    fun shift_window_right(crsn: &mut CRSN) {
        let index = BitVector::longest_set_sequence_starting_at(&crsn.slots, 0);

        // if there is no run of set bits return early
        if (index == 0) return;
        BitVector::shift_left(&mut crsn.slots, index);
        crsn.min_nonce = crsn.min_nonce + index;
    }


    /***************************************************************************/
    // tests
    /***************************************************************************/

   #[test(a=@0x0)]
   fun publish_exists_after_small_size(a: signer)
   acquires CRSN {
       let addr = Signer::address_of(&a);

       publish(&a, 0, 10);
       assert(exists<CRSN>(addr), 0);
       let crsn = borrow_global<CRSN>(addr);
       assert(crsn.min_nonce == 0, 1);
       assert(BitVector::length(&crsn.slots) == 10, 2);
   }

   #[test(a=@0x0)]
   fun publish_exists_after_medium_size(a: signer)
   acquires CRSN {
       let addr = Signer::address_of(&a);

       publish(&a, 20, 128);
       assert(exists<CRSN>(addr), 0);
       let crsn = borrow_global<CRSN>(addr);
       assert(crsn.min_nonce == 20, 1);
       assert(BitVector::length(&crsn.slots) == 128, 2);
   }

   #[test(a=@0x0)]
   fun publish_exists_after_large_size(a: signer)
   acquires CRSN {
       let addr = Signer::address_of(&a);

       publish(&a, 505, 1033);
       assert(exists<CRSN>(addr), 0);
       let crsn = borrow_global<CRSN>(addr);
       assert(crsn.min_nonce == 505, 1);
       assert(BitVector::length(&crsn.slots) == 1033, 2);
   }


   #[test(a=@0x0)]
   #[expected_failure(abort_code = 257)]
   fun double_publish(a: signer) {
       publish(&a, 0, 10);
       publish(&a, 0, 10);
   }

   #[test(a=@0x0)]
   #[expected_failure(abort_code = 519)]
   fun publish_zero_size(a: signer) {
       publish(&a, 10, 0);
   }

   #[test(a=@0x0)]
   fun test_has_crsn(a: signer) {
       let addr = Signer::address_of(&a);
       assert(!has_crsn(addr), 0);
       publish(&a, 0, 10);
       assert(has_crsn(addr), 1);
   }

    #[test(a=@0x0)]
    #[expected_failure(abort_code = 1)]
    fun record_no_crsn(a: signer)
    acquires CRSN {
        record(&a, 0);
    }

    #[test(a=@0x0)]
    fun record_too_high_low_accept(a: signer)
    acquires CRSN {
       publish(&a, 100, 10);
       assert(!record(&a, 110), 0);
       assert(!record(&a, 111), 1);
       assert(!record(&a, 99), 2);
       assert(record(&a, 100), 3);
       assert(record(&a, 109), 4);
       assert(!record(&a, 100), 5);
       assert(!record(&a, 109), 6);
    }

    #[test(a=@0x0)]
    fun prevent_replay(a: signer)
    acquires CRSN {
       publish(&a, 100, 10);
       assert(record(&a, 101), 0);
       assert(!record(&a, 101), 1);
    }

    #[test(a=@0x0)]
    fun prevent_replay_with_shift(a: signer)
    acquires CRSN {
       publish(&a, 100, 10);
       assert(record(&a, 100), 0);
       assert(!record(&a, 100), 1);
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 101, 2);
    }

    #[test(a=@0x0)]
    fun multiple_shifts_of_window(a: signer)
    acquires CRSN {
       publish(&a, 100, 10);
       assert(record(&a, 101), 0);
       assert(record(&a, 102), 0);
       assert(record(&a, 103), 0);

       assert(record(&a, 106), 0);
       assert(record(&a, 107), 0);
       assert(record(&a, 108), 0);

       // The window should not have shifted
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 100, 1);
       assert(record(&a, 100), 0);
       // The window should now shift until it gets stuck on 104
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 104, 1);
       assert(record(&a, 104), 0);
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 105, 1);
       assert(record(&a, 105), 0);
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 109, 1);

       // Now make sure that the window has shifted and opened-up higher slots
       assert(record(&a, 110), 0);
       assert(record(&a, 111), 0);
       assert(record(&a, 112), 0);
       assert(record(&a, 113), 0);
       assert(record(&a, 114), 0);
       assert(record(&a, 115), 0);
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 109, 1);
       assert(record(&a, 109), 0);
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 116, 1);
    }

    #[test(a=@0x0)]
    fun force_expire_single_and_zero(a: signer)
    acquires CRSN {
       publish(&a, 100, 10);
       force_expire(&a, 0);
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 100, 1);
       force_expire(&a, 1);
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 101, 1);
    }

    #[test(a=@0x0)]
    fun force_expire_shift_over_set_bits(a: signer)
    acquires CRSN {
       publish(&a, 0, 100);
       assert(record(&a, 1), 0);
       assert(record(&a, 2), 0);
       assert(record(&a, 3), 0);
       force_expire(&a, 1);
       assert(borrow_global<CRSN>(Signer::address_of(&a)).min_nonce == 4, 1);
    }

    #[test(a=@0x0)]
    fun force_expire_past_set_bits(a: signer)
    acquires CRSN {
       publish(&a, 0, 100);
       assert(record(&a, 1), 0);
       assert(record(&a, 2), 0);
       assert(record(&a, 3), 0);
       force_expire(&a, 15);
       let crsn = borrow_global<CRSN>(Signer::address_of(&a));
       assert(crsn.min_nonce == 15, 1);
       let i = 0;
       let len = 100;

       while (i < len) {
           assert(!BitVector::is_index_set(&crsn.slots, i), 2);
           i = i + 1;
       }
    }

    #[test(a=@0x0)]
    fun force_expire_past_window_size(a: signer)
    acquires CRSN {
       publish(&a, 0, 100);
       assert(record(&a, 1), 0);
       assert(record(&a, 2), 0);
       assert(record(&a, 3), 0);
       force_expire(&a, 10000);
       let crsn = borrow_global<CRSN>(Signer::address_of(&a));
       assert(crsn.min_nonce == 10000, 1);
       let i = 0;
       let len = 100;

       while (i < len) {
           assert(!BitVector::is_index_set(&crsn.slots, i), 2);
           i = i + 1;
       }
    }
}
}
