// Number of bits shifted >= total number of bits in the number.
script {
fun main() {
    0u8 << 8u8;
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    0u64 << 64u8;
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    0u128 << 128u8;
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    0u8 >> 8u8;
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    0u64 >> 64u8;
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    0u128 >> 128u8;
}
}
// check: ARITHMETIC_ERROR



// Shifting 0 results in 0.
//! new-transaction
script {
fun main() {
    assert(0u8 << 4u8 == 0u8, 1000);
    assert(0u64 << 1u8 == 0u64, 1001);
    assert(0u128 << 127u8 == 0u128, 1002);

    assert(0u8 >> 4u8 == 0u8, 1100);
    assert(0u64 >> 1u8 == 0u64, 1101);
    assert(0u128 >> 127u8 == 0u128, 1102);
}
}
// check: "Keep(EXECUTED)"



// Shifting by 0 bits results in the same number.
//! new-transaction
script {
fun main() {
    assert(100u8 << 0u8 == 100u8, 2000);
    assert(43u64 << 0u8 == 43u64, 2001);
    assert(57348765484584586725315342563424u128 << 0u8 == 57348765484584586725315342563424u128, 2002);

    assert(100u8 >> 0u8 == 100u8, 2100);
    assert(43u64 >> 0u8 == 43u64, 2101);
    assert(57348765484584586725315342563424u128 >> 0u8 == 57348765484584586725315342563424u128, 2102);
}
}
// check: "Keep(EXECUTED)"



// shl/shr by 1 equivalent to mul/div by 2.
//! new-transaction
script {
fun main() {
    assert(1u8 << 1u8 == 2u8, 3000);
    assert(7u64 << 1u8 == 14u64, 3001);
    assert(1000u128 << 1u8 == 2000u128, 3002);

    assert(1u8 >>1u8 == 0u8, 3100);
    assert(7u64 >> 1u8 == 3u64, 3101);
    assert(1000u128 >> 1u8 == 500u128, 3102);
}
}
// check: "Keep(EXECUTED)"



// Underflowing results in 0.
//! new-transaction
script {
fun main() {
    assert(1234u64 >> 63u8 == 0u64, 4000);
    assert(3u8 >> 5u8 == 0u8, 4001);
    assert(43152365326753472145312542634526753u128 >> 127u8 == 0u128, 4002);
}
}
// check: "Keep(EXECUTED)"



// Overflowing results are truncated.
//! new-transaction
script {
fun main() {
    assert(7u8 << 7u8 == 128u8, 5000);
    assert(7u64 << 62u8 == 13835058055282163712u64, 5001);
    assert(2u128 << 127u8 == 0u128, 5002);
}
}
// check: "Keep(EXECUTED)"



// Some random tests.
//! new-transaction
script {
fun main() {
    assert(54u8 << 3u8 == 176u8, 6000);
    assert(5u8 << 2u8 == 20u8, 6001);
    assert(124u8 << 5u8 == 128u8, 6002);

    assert(326348456u64 << 13u8 == 2673446551552u64, 6100);
    assert(218u64 << 30u8 == 234075717632u64, 6101);
    assert(345325745376476456u64 << 47u8 == 2203386117691015168u64, 6102);

    assert(95712896789423756892376u128 << 4u8 == 1531406348630780110278016u128, 6200);
    assert(8629035907847368941279654523567912314u128 << 77u8 == 317056859699765342273530379836650946560u128, 6201);
    assert(5742389768935678297185789157531u128 << 10u8 == 5880207123390134576318248097311744u128, 6202);
    assert(295429678238907658936718926478967892769u128 << 83u8 == 78660438169199498567214234129963941888u128, 6203);
}
}
// check: "Keep(EXECUTED)"
