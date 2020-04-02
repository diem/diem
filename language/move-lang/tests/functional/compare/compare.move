// Tests for polymorphic comparison in Move

use 0x0::Compare;
use 0x0::LCS;
use 0x0::Transaction;

fun main() {
    // TODO: replace with constants once the source lang has them
    let EQUAL = 0u8;
    let LESS_THAN = 1u8;
    let GREATER_THAN = 2u8;

    // equality of simple types
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&true), &LCS::to_bytes(&true)) == EQUAL, 8001);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u8), &LCS::to_bytes(&1u8)) == EQUAL, 8002);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1), &LCS::to_bytes(&1)) == EQUAL, 8003);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u128), &LCS::to_bytes(&1u128)) == EQUAL, 8004);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x1), &LCS::to_bytes(&0x1)) == EQUAL, 8005);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"1"), &LCS::to_bytes(&x"1")) == EQUAL, 8006);

    // inequality of simple types
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&true), &LCS::to_bytes(&false)) != EQUAL, 8007);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u8), &LCS::to_bytes(&0u8)) != EQUAL, 8008);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1), &LCS::to_bytes(&0)) != EQUAL, 8009);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u128), &LCS::to_bytes(&0u128)) != EQUAL, 8010);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x1), &LCS::to_bytes(&0x0)) != EQUAL, 8011);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"1"), &LCS::to_bytes(&x"0")) != EQUAL, 8012);

    // less than for types with a natural ordering exposed via bytecode operations
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&false), &LCS::to_bytes(&true)) == LESS_THAN, 8013);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0u8), &LCS::to_bytes(&1u8)) == LESS_THAN, 8014);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0), &LCS::to_bytes(&1)) == LESS_THAN, 8015);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0u128), &LCS::to_bytes(&1u128)) == LESS_THAN, 8016);

    // less then for types without a natural ordering exposed by bytecode operations
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x0), &LCS::to_bytes(&0x1)) == LESS_THAN, 8017); // sensible
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x01), &LCS::to_bytes(&0x10)) == LESS_THAN, 8018); // sensible
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x100), &LCS::to_bytes(&0x001)) == LESS_THAN, 8019); // potentially confusing
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"0"), &LCS::to_bytes(&x"1")) == LESS_THAN, 8020); // sensible
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"01"), &LCS::to_bytes(&x"10")) == LESS_THAN, 8021); // sensible
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"000"), &LCS::to_bytes(&x"01")) == LESS_THAN, 8022); //
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"100"), &LCS::to_bytes(&x"001")) == LESS_THAN, 8023); // potentially confusing

    // greater than for types with a natural ordering exposed by bytecode operations
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&true), &LCS::to_bytes(&false)) == GREATER_THAN, 8024);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u8), &LCS::to_bytes(&0u8)) == GREATER_THAN, 8025);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1), &LCS::to_bytes(&0)) == GREATER_THAN, 8026);
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u128), &LCS::to_bytes(&0u128)) == GREATER_THAN, 8027);

    // greater than for types without a natural ordering exposed by by bytecode operations
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x1), &LCS::to_bytes(&0x0)) == GREATER_THAN, 8028); // sensible
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x10), &LCS::to_bytes(&0x01)) == GREATER_THAN, 8029); // sensible
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x001), &LCS::to_bytes(&0x100)) == GREATER_THAN, 8030); // potentially confusing
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"1"), &LCS::to_bytes(&x"0")) == GREATER_THAN, 8031); // sensible
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"10"), &LCS::to_bytes(&x"01")) == GREATER_THAN, 8032); // sensible
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"01"), &LCS::to_bytes(&x"000")) == GREATER_THAN, 8033); // sensible
    Transaction::assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"001"), &LCS::to_bytes(&x"100")) == GREATER_THAN, 8034); // potentially confusing
}

// check: EXECUTED
