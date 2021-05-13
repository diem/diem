// Tests for polymorphic comparison in Move
#[test_only]
module Std::CompareTests {
    use Std::Compare;
    use Std::BCS;

    const EQUAL: u8 = 0;
    const LESS_THAN: u8 = 1;
    const GREATER_THAN: u8 = 2;


    #[test]
    fun equality_of_simple_types() {
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&true), &BCS::to_bytes(&true)) == EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u8), &BCS::to_bytes(&1u8)) == EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1), &BCS::to_bytes(&1)) == EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u128), &BCS::to_bytes(&1u128)) == EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x1), &BCS::to_bytes(&@0x1)) == EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"01")) == EQUAL, 0);
    }

    #[test]
    fun inequality_of_simple_types() {
        // inequality of simple types
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&true), &BCS::to_bytes(&false)) != EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u8), &BCS::to_bytes(&0u8)) != EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1), &BCS::to_bytes(&0)) != EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u128), &BCS::to_bytes(&0u128)) != EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x1), &BCS::to_bytes(&@0x0)) != EQUAL, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"00")) != EQUAL, 0);
    }

    #[test]
    fun less_than_with_natural_ordering() {
        // less than for types with a natural ordering exposed via bytecode operations
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&false), &BCS::to_bytes(&true)) == LESS_THAN, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&0u8), &BCS::to_bytes(&1u8)) == LESS_THAN, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&0), &BCS::to_bytes(&1)) == LESS_THAN, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&0u128), &BCS::to_bytes(&1u128)) == LESS_THAN, 0);
    }

    #[test]
    fun less_than_without_natural_ordering() {
        // less then for types without a natural ordering exposed by bytecode operations
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x0), &BCS::to_bytes(&@0x1)) == LESS_THAN, 0); // sensible
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x01), &BCS::to_bytes(&@0x10)) == LESS_THAN, 0); // sensible
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x100), &BCS::to_bytes(&@0x001)) == LESS_THAN, 0); // potentially confusing
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"00"), &BCS::to_bytes(&x"01")) == LESS_THAN, 0); // sensible
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"10")) == LESS_THAN, 0); // sensible
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"0000"), &BCS::to_bytes(&x"01")) == LESS_THAN, 0); //
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"0100"), &BCS::to_bytes(&x"0001")) == LESS_THAN, 0); // potentially confusing
    }

    #[test]
    fun greater_than_with_natural_ordering() {
        // greater than for types with a natural ordering exposed by bytecode operations
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&true), &BCS::to_bytes(&false)) == GREATER_THAN, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u8), &BCS::to_bytes(&0u8)) == GREATER_THAN, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1), &BCS::to_bytes(&0)) == GREATER_THAN, 0);
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u128), &BCS::to_bytes(&0u128)) == GREATER_THAN, 0);
    }

    #[test]
    fun greater_than_without_natural_ordering() {
        // greater than for types without a natural ordering exposed by by bytecode operations
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x1), &BCS::to_bytes(&@0x0)) == GREATER_THAN, 0); // sensible
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x10), &BCS::to_bytes(&@0x01)) == GREATER_THAN, 0); // sensible
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x001), &BCS::to_bytes(&@0x100)) == GREATER_THAN, 0); // potentially confusing
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"00")) == GREATER_THAN, 0); // sensible
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"10"), &BCS::to_bytes(&x"01")) == GREATER_THAN, 0); // sensible
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"0000")) == GREATER_THAN, 0); // sensible
        assert(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"0001"), &BCS::to_bytes(&x"0100")) == GREATER_THAN, 0); // potentially confusing
    }
}
