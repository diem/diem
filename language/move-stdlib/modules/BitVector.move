address 0x1 {
module BitVector {
    use 0x1::Vector;
    use 0x1::Errors;

    /// The provided index is out of bounds
    const EINDEX: u64 = 0;

    const WORD_SIZE: u64 = 64;

    struct BitVector has copy, drop, store {
        length: u64,
        bit_field: vector<u64>,
    }

    public fun new(length: u64): BitVector {
        let num_words = (length + (WORD_SIZE - 1)) /  WORD_SIZE;
        let bit_field = Vector::empty();
        while (num_words > 0) {
            Vector::push_back(&mut bit_field, 0u64);
            num_words = num_words - 1;
        };

        BitVector {
            length,
            bit_field,
        }
    }

    /// Set the bit at `bit_index` in the `bitvector` regardless of its previous state.
    public fun set(bitvector: &mut BitVector, bit_index: u64) {
        assert(bit_index < bitvector.length, Errors::invalid_argument(EINDEX));
        let (inner_index, inner) = bit_index(bitvector, bit_index);
        *inner = *inner | 1u64 << (inner_index as u8);
    }

    /// Unset the bit at `bit_index` in the `bitvector` regardless of its previous state.
    public fun unset(bitvector: &mut BitVector, bit_index: u64) {
        assert(bit_index < bitvector.length, Errors::invalid_argument(EINDEX));
        let (inner_index, inner) = bit_index(bitvector, bit_index);
        // Having negation would be nice here...
        *inner = *inner ^ (*inner & (1u64 << (inner_index as u8)));
    }

    /// Shift the `bitvector` left by `amount`, `amount` must be less than the
    /// bitvector's length.
    public fun shift_left(bitvector: &mut BitVector, amount: u64) {
        if (amount >= bitvector.length) {
           let len = Vector::length(&bitvector.bit_field);
           let i = 0;
           while (i < len) {
               let elem = Vector::borrow_mut(&mut bitvector.bit_field, i);
               *elem = 0;
               i = i + 1;
           };
        } else {
            let i = amount;

            while (i < bitvector.length) {
                if (is_index_set(bitvector, i)) set(bitvector, i - amount)
                else unset(bitvector, i - amount);
                i = i + 1;
            };

            i = bitvector.length - amount;

            while (i < bitvector.length) {
                unset(bitvector, i);
                i = i + 1;
            };
        }
    }

    /// Return the value of the bit at `bit_index` in the `bitvector`. `true`
    /// represents "1" and `false` represents a 0
    public fun is_index_set(bitvector: &BitVector, bit_index: u64): bool {
        assert(bit_index < bitvector.length, Errors::invalid_argument(EINDEX));
        let inner = Vector::borrow(&bitvector.bit_field, bit_index / WORD_SIZE);
        let inner_index = bit_index % WORD_SIZE;
        *inner & (1 << (inner_index as u8)) != 0
    }

    /// Return the length (number of usable bits) of this bitvector
    public fun length(bitvector: &BitVector): u64 {
        bitvector.length
    }

    /// Returns the length of the longest sequence of set bits starting at (and
    /// including) `start_index` in the `bitvector`. If there is no such
    /// sequence, then `0` is returned.
    public fun longest_set_sequence_starting_at(bitvector: &BitVector, start_index: u64): u64 {
        assert(start_index < bitvector.length, Errors::invalid_argument(EINDEX));
        let index = start_index;

        // Find the greatest index in the vector such that all indices less than it are set.
        while (index < bitvector.length) {
            if (!is_index_set(bitvector, index)) break;
            index = index + 1;
        };

        index - start_index
    }

    /// Return the larger containing u64, and the bit index within that u64
    /// for `index` w.r.t. `bitvector`.
    fun bit_index(bitvector: &mut BitVector, index: u64): (u64, &mut u64) {
        assert(index < bitvector.length, Errors::invalid_argument(EINDEX));
        (index % WORD_SIZE, Vector::borrow_mut(&mut bitvector.bit_field, index / WORD_SIZE))
    }

    spec module {
        pragma verify = false;
    }

    /***************************************************************************/
    // tests
    /***************************************************************************/

    #[test_only]
    fun test_bitvector_set_unset_of_size(k: u64) {
        let bitvector = new(k);
        let index = 0;
        while (index < k) {
            set(&mut bitvector, index);
            assert(is_index_set(&bitvector, index), 0);
            index = index + 1;
            let index_to_right = index;
            while (index_to_right < k) {
                assert(!is_index_set(&bitvector, index_to_right), 1);
                index_to_right = index_to_right + 1;
            };
        };

        // now go back down unsetting
        index = 0;

        while (index < k) {
            unset(&mut bitvector, index);
            assert(!is_index_set(&bitvector, index), 0);
            index = index + 1;
            let index_to_right = index;
            while (index_to_right < k) {
                assert(is_index_set(&bitvector, index_to_right), 1);
                index_to_right = index_to_right + 1;
            };
        };
    }

    #[test]
    #[expected_failure(abort_code = 7)]
    fun set_bit_out_of_bounds() {
        let bitvector = new(WORD_SIZE);
        set(&mut bitvector, WORD_SIZE);
    }

    #[test]
    #[expected_failure(abort_code = 7)]
    fun unset_bit_out_of_bounds() {
        let bitvector = new(WORD_SIZE);
        unset(&mut bitvector, WORD_SIZE);
    }

    #[test]
    #[expected_failure(abort_code = 7)]
    fun index_bit_out_of_bounds() {
        let bitvector = new(WORD_SIZE);
        is_index_set(&mut bitvector, WORD_SIZE);
    }

    #[test]
    fun test_set_bit_and_index_basic() {
        test_bitvector_set_unset_of_size(8)
    }

    #[test]
    fun test_set_bit_and_index_odd_size() {
        test_bitvector_set_unset_of_size(300)
    }

    #[test]
    fun longest_sequence_no_set_zero_index() {
        let bitvector = new(100);
        assert(longest_set_sequence_starting_at(&bitvector, 0) == 0, 0);
    }

    #[test]
    fun longest_sequence_one_set_zero_index() {
        let bitvector = new(100);
        set(&mut bitvector, 1);
        assert(longest_set_sequence_starting_at(&bitvector, 0) == 0, 0);
    }

    #[test]
    fun longest_sequence_no_set_nonzero_index() {
        let bitvector = new(100);
        assert(longest_set_sequence_starting_at(&bitvector, 51) == 0, 0);
    }

    #[test]
    fun longest_sequence_two_set_nonzero_index() {
        let bitvector = new(100);
        set(&mut bitvector, 50);
        set(&mut bitvector, 52);
        assert(longest_set_sequence_starting_at(&bitvector, 51) == 0, 0);
    }

    #[test]
    fun longest_sequence_with_break() {
        let bitvector = new(100);
        let i = 0;
        while (i < 20) {
            set(&mut bitvector, i);
            i = i + 1;
        };
        // create a break in the run
        i = i + 1;
        while (i < 100) {
            set(&mut bitvector, i);
            i = i + 1;
        };
        assert(longest_set_sequence_starting_at(&bitvector, 0) == 20, 0);
        assert(longest_set_sequence_starting_at(&bitvector, 20) == 0, 0);
        assert(longest_set_sequence_starting_at(&bitvector, 21) == 100 - 21, 0);
    }

    #[test]
    fun test_shift_left() {
        let bitlen = 133;
        let bitvector = new(bitlen);

        let i = 0;
        while (i < bitlen) {
            set(&mut bitvector, i);
            i = i + 1;
        };

        i = bitlen - 1;
        while (i > 0) {
            assert(is_index_set(&bitvector, i), 0);
            shift_left(&mut bitvector, 1);
            assert(!is_index_set(&bitvector,  i), 1);
            i = i - 1;
        };
    }

    #[test]
    fun test_shift_left_specific_amount() {
        let bitlen = 300;
        let shift_amount = 133;
        let bitvector = new(bitlen);

        set(&mut bitvector, 201);
        assert(is_index_set(&bitvector, 201), 0);

        shift_left(&mut bitvector, shift_amount);
        assert(is_index_set(&bitvector, 201 - shift_amount), 1);
        assert(!is_index_set(&bitvector, 201), 2);

        // Make sure this shift clears all the bits
        shift_left(&mut bitvector, bitlen  - 1);

        let i = 0;
        while (i < bitlen) {
            assert(!is_index_set(&bitvector, i), 3);
            i = i + 1;
        }
    }

    #[test]
    fun test_shift_left_specific_amount_to_unset_bit() {
        let bitlen = 50;
        let chosen_index = 24;
        let shift_amount = 3;
        let bitvector = new(bitlen);

        let i = 0;

        while (i < bitlen) {
            set(&mut bitvector, i);
            i = i + 1;
        };

        unset(&mut bitvector, chosen_index);
        assert(!is_index_set(&bitvector, chosen_index), 0);

        shift_left(&mut bitvector, shift_amount);

        i = 0;

        while (i < bitlen) {
            // only chosen_index - shift_amount and the remaining bits should be unset
            if ((i == chosen_index - shift_amount) || (i >= bitlen - shift_amount)) {
                assert(!is_index_set(&bitvector, i), 1);
            } else {
                assert(is_index_set(&bitvector, i), 2);
            };
            i = i + 1;
        }
    }

    #[test]
    fun shift_left_at_size() {
        let bitlen = 133;
        let bitvector = new(bitlen);

        let i = 0;
        while (i < bitlen) {
            set(&mut bitvector, i);
            i = i + 1;
        };

        shift_left(&mut bitvector, bitlen - 1);
        i = bitlen - 1;
        while (i > 0) {
            assert(!is_index_set(&bitvector,  i), 1);
            i = i - 1;
        };
    }

    #[test]
    fun shift_left_more_than_size() {
        let bitlen = 133;
        let bitvector = new(bitlen);
        set(&mut bitvector, 132);
        shift_left(&mut bitvector, bitlen);
        let i = 0;
        while (i < bitlen) {
            assert(!is_index_set(&bitvector, i), 3);
            i = i + 1;
        }
    }

    #[test]
    fun empty_bitvector() {
        let bitvector = new(0);
        assert(bitvector.length == 0, 0);
    }

    #[test]
    fun single_bit_bitvector() {
        let bitvector = new(1);
        assert(bitvector.length == 1, 0);
    }
}
}
