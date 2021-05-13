#[test_only]
module Std::BitVectorTests {
    use Std::BitVector;

    #[test_only]
    fun test_bitvector_set_unset_of_size(k: u64) {
        let bitvector = BitVector::new(k);
        let index = 0;
        while (index < k) {
            BitVector::set(&mut bitvector, index);
            assert(BitVector::is_index_set(&bitvector, index), 0);
            index = index + 1;
            let index_to_right = index;
            while (index_to_right < k) {
                assert(!BitVector::is_index_set(&bitvector, index_to_right), 1);
                index_to_right = index_to_right + 1;
            };
        };
        // now go back down unsetting
        index = 0;

        while (index < k) {
            BitVector::unset(&mut bitvector, index);
            assert(!BitVector::is_index_set(&bitvector, index), 0);
            index = index + 1;
            let index_to_right = index;
            while (index_to_right < k) {
                assert(BitVector::is_index_set(&bitvector, index_to_right), 1);
                index_to_right = index_to_right + 1;
            };
        };
    }

    #[test]
    #[expected_failure(abort_code = 0)]
    fun set_bit_out_of_bounds() {
        let bitvector = BitVector::new(BitVector::word_size());
        BitVector::set(&mut bitvector, BitVector::word_size());
    }

    #[test]
    #[expected_failure(abort_code = 0)]
    fun unset_bit_out_of_bounds() {
        let bitvector = BitVector::new(BitVector::word_size());
        BitVector::unset(&mut bitvector, BitVector::word_size());
    }

    #[test]
    #[expected_failure(abort_code = 0)]
    fun index_bit_out_of_bounds() {
        let bitvector = BitVector::new(BitVector::word_size());
        BitVector::is_index_set(&mut bitvector, BitVector::word_size());
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
    fun test_shift_left() {
        let bitlen = 133;
        let bitvector = BitVector::new(bitlen);

        let i = 0;
        while (i < bitlen) {
            BitVector::set(&mut bitvector, i);
            i = i + 1;
        };

        i = bitlen - 1;
        while (i > 0) {
            assert(BitVector::is_index_set(&bitvector, i), 0);
            BitVector::shift_left(&mut bitvector, 1);
            assert(!BitVector::is_index_set(&bitvector,  i), 1);
            i = i - 1;
        };
    }

    #[test]
    fun test_shift_left_specific_amount() {
        let bitlen = 300;
        let shift_amount = 133;
        let bitvector = BitVector::new(bitlen);

        BitVector::set(&mut bitvector, 201);
        assert(BitVector::is_index_set(&bitvector, 201), 0);

        BitVector::shift_left(&mut bitvector, shift_amount);
        assert(BitVector::is_index_set(&bitvector, 201 - shift_amount), 1);
        assert(!BitVector::is_index_set(&bitvector, 201), 2);

        // Make sure this shift clears all the bits
        BitVector::shift_left(&mut bitvector, bitlen  - 1);

        let i = 0;
        while (i < bitlen) {
            assert(!BitVector::is_index_set(&bitvector, i), 3);
            i = i + 1;
        }
    }

    #[test]
    fun test_shift_left_specific_amount_to_unset_bit() {
        let bitlen = 50;
        let chosen_index = 24;
        let shift_amount = 3;
        let bitvector = BitVector::new(bitlen);

        let i = 0;

        while (i < bitlen) {
            BitVector::set(&mut bitvector, i);
            i = i + 1;
        };

        BitVector::unset(&mut bitvector, chosen_index);
        assert(!BitVector::is_index_set(&bitvector, chosen_index), 0);

        BitVector::shift_left(&mut bitvector, shift_amount);

        i = 0;

        while (i < bitlen) {
            // only chosen_index - shift_amount and the remaining bits should be BitVector::unset
            if ((i == chosen_index - shift_amount) || (i >= bitlen - shift_amount)) {
                assert(!BitVector::is_index_set(&bitvector, i), 1);
            } else {
                assert(BitVector::is_index_set(&bitvector, i), 2);
            };
            i = i + 1;
        }
    }

    #[test]
    fun shift_left_at_size() {
        let bitlen = 133;
        let bitvector = BitVector::new(bitlen);

        let i = 0;
        while (i < bitlen) {
            BitVector::set(&mut bitvector, i);
            i = i + 1;
        };

        BitVector::shift_left(&mut bitvector, bitlen - 1);
        i = bitlen - 1;
        while (i > 0) {
            assert(!BitVector::is_index_set(&bitvector,  i), 1);
            i = i - 1;
        };
    }

    #[test]
    #[expected_failure(abort_code = 0)]
    fun shift_left_more_than_size() {
        let bitlen = 133;
        let bitvector = BitVector::new(bitlen);
        BitVector::shift_left(&mut bitvector, bitlen);
    }

    #[test]
    fun empty_bitvector() {
        let bitvector = BitVector::new(0);
        assert(BitVector::length(&bitvector) == 0, 0);
    }

    #[test]
    fun single_bit_bitvector() {
        let bitvector = BitVector::new(1);
        assert(BitVector::length(&bitvector) == 1, 0);
    }
}
