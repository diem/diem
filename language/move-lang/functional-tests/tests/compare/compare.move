// Tests for polymorphic comparison in Move
script {
use 0x0::Compare;
use 0x0::LCS;

fun main() {
    // TODO: replace with constants once the source lang has them
    let equal = 0u8;
    let less_than = 1u8;
    let greater_than = 2u8;

    // equality of simple types
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&true), &LCS::to_bytes(&true)) == equal, 8001);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u8), &LCS::to_bytes(&1u8)) == equal, 8002);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1), &LCS::to_bytes(&1)) == equal, 8003);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u128), &LCS::to_bytes(&1u128)) == equal, 8004);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x1), &LCS::to_bytes(&0x1)) == equal, 8005);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"1"), &LCS::to_bytes(&x"1")) == equal, 8006);

    // inequality of simple types
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&true), &LCS::to_bytes(&false)) != equal, 8007);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u8), &LCS::to_bytes(&0u8)) != equal, 8008);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1), &LCS::to_bytes(&0)) != equal, 8009);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u128), &LCS::to_bytes(&0u128)) != equal, 8010);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x1), &LCS::to_bytes(&0x0)) != equal, 8011);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"1"), &LCS::to_bytes(&x"0")) != equal, 8012);

    // less than for types with a natural ordering exposed via bytecode operations
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&false), &LCS::to_bytes(&true)) == less_than, 8013);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0u8), &LCS::to_bytes(&1u8)) == less_than, 8014);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0), &LCS::to_bytes(&1)) == less_than, 8015);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0u128), &LCS::to_bytes(&1u128)) == less_than, 8016);

    // less then for types without a natural ordering exposed by bytecode operations
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x0), &LCS::to_bytes(&0x1)) == less_than, 8017); // sensible
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x01), &LCS::to_bytes(&0x10)) == less_than, 8018); // sensible
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x100), &LCS::to_bytes(&0x001)) == less_than, 8019); // potentially confusing
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"0"), &LCS::to_bytes(&x"1")) == less_than, 8020); // sensible
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"01"), &LCS::to_bytes(&x"10")) == less_than, 8021); // sensible
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"000"), &LCS::to_bytes(&x"01")) == less_than, 8022); //
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"100"), &LCS::to_bytes(&x"001")) == less_than, 8023); // potentially confusing

    // greater than for types with a natural ordering exposed by bytecode operations
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&true), &LCS::to_bytes(&false)) == greater_than, 8024);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u8), &LCS::to_bytes(&0u8)) == greater_than, 8025);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1), &LCS::to_bytes(&0)) == greater_than, 8026);
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&1u128), &LCS::to_bytes(&0u128)) == greater_than, 8027);

    // greater than for types without a natural ordering exposed by by bytecode operations
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x1), &LCS::to_bytes(&0x0)) == greater_than, 8028); // sensible
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x10), &LCS::to_bytes(&0x01)) == greater_than, 8029); // sensible
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&0x001), &LCS::to_bytes(&0x100)) == greater_than, 8030); // potentially confusing
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"1"), &LCS::to_bytes(&x"0")) == greater_than, 8031); // sensible
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"10"), &LCS::to_bytes(&x"01")) == greater_than, 8032); // sensible
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"01"), &LCS::to_bytes(&x"000")) == greater_than, 8033); // sensible
    assert(Compare::cmp_lcs_bytes(&LCS::to_bytes(&x"001"), &LCS::to_bytes(&x"100")) == greater_than, 8034); // potentially confusing
}
}
// check: EXECUTED
