module Std::BitVector {
    use Std::Vector;

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
        assert(bit_index < bitvector.length, EINDEX);
        let (inner_index, inner) = bit_index(bitvector, bit_index);
        *inner = *inner | 1u64 << (inner_index as u8);
    }

    /// Unset the bit at `bit_index` in the `bitvector` regardless of its previous state.
    public fun unset(bitvector: &mut BitVector, bit_index: u64) {
        assert(bit_index < bitvector.length, EINDEX);
        let (inner_index, inner) = bit_index(bitvector, bit_index);
        // Having negation would be nice here...
        *inner = *inner ^ (*inner & (1u64 << (inner_index as u8)));
    }

    /// Shift the `bitvector` left by `amount`, `amount` must be less than the
    /// bitvector's length.
    public fun shift_left(bitvector: &mut BitVector, amount: u64) {
        assert(amount < bitvector.length, EINDEX);
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

    /// Return the value of the bit at `bit_index` in the `bitvector`. `true`
    /// represents "1" and `false` represents a 0
    public fun is_index_set(bitvector: &BitVector, bit_index: u64): bool {
        assert(bit_index < bitvector.length, EINDEX);
        let inner = Vector::borrow(&bitvector.bit_field, bit_index / WORD_SIZE);
        let inner_index = bit_index % WORD_SIZE;
        *inner & (1 << (inner_index as u8)) != 0
    }

    /// Return the length (number of bits) in the bitvector.
    public fun length(bitvector: &BitVector): u64 {
        bitvector.length
    }

    /// Return the larger containing u64, and the bit index within that u64
    /// for `index` w.r.t. `bitvector`.
    fun bit_index(bitvector: &mut BitVector, index: u64): (u64, &mut u64) {
        assert(index < bitvector.length, EINDEX);
        (index % WORD_SIZE, Vector::borrow_mut(&mut bitvector.bit_field, index / WORD_SIZE))
    }

    #[test_only]
    public fun word_size(): u64 {
        WORD_SIZE
    }

    spec module {
        pragma verify = false;
    }
}
