// Casting to u8.
script {
fun main() {
    // 0 remains unchanged.
    assert((0u8 as u8) == 0u8, 1000);
    assert((0u64 as u8) == 0u8, 1001);
    assert((0u128 as u8) == 0u8, 1002);

    // Random small number unchanged.
    assert((21u8 as u8) == 21u8, 1100);
    assert((21u64 as u8) == 21u8, 1101);
    assert((21u128 as u8) == 21u8, 1102);

    // Max representable values remain unchanged.
    assert((255u8 as u8) == 255u8, 1200);
    assert((255u64 as u8) == 255u8, 1201);
    assert((255u128 as u8) == 255u8, 1202);
}
}
// check: EXECUTED

// Casting to u64.
//! new-transaction
script {
fun main() {
    // 0 remains unchanged.
    assert((0u8 as u64) == 0u64, 2000);
    assert((0u64 as u64) == 0u64, 2001);
    assert((0u128 as u64) == 0u64, 2002);

    // Random small number unchanged.
    assert((21u8 as u64) == 21u64, 2100);
    assert((21u64 as u64) == 21u64, 2101);
    assert((21u128 as u64) == 21u64, 2102);

    // Max representable values remain unchanged.
    assert((255u8 as u64) == 255u64, 2200);
    assert((18446744073709551615u64 as u64) == 18446744073709551615u64, 2201);
    assert((18446744073709551615u128 as u64) == 18446744073709551615u64, 2202);
}
}
// check: EXECUTED

// Casting to u128.
//! new-transaction
script {
fun main() {
    // 0 remains unchanged.
    assert((0u8 as u128) == 0u128, 3000);
    assert((0u64 as u128) == 0u128, 3001);
    assert((0u128 as u128) == 0u128, 3002);

    // Random small number unchanged.
    assert((21u8 as u128) == 21u128, 3100);
    assert((21u64 as u128) == 21u128, 3101);
    assert((21u128 as u128) == 21u128, 3102);

    // Max representable values remain unchanged.
    assert((255u8 as u128) == 255u128, 3200);
    assert((18446744073709551615u64 as u128) == 18446744073709551615u128, 3201);
    assert((340282366920938463463374607431768211455u128 as u128) == 340282366920938463463374607431768211455u128, 3202);
}
}
// check: EXECUTED


// Casting to u8, overflowing.
//! new-transaction
script {
fun main() {
    (256u64 as u8);
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    (303u64 as u8);
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    (256u128 as u8);
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    (56432u128 as u8);
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    (18446744073709551615u64 as u8);
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    (340282366920938463463374607431768211455u128 as u8);
}
}
// check: ARITHMETIC_ERROR


// Casting to u64, overflowing.
//! new-transaction
script {
fun main() {
    (18446744073709551616u128 as u64);
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    (18446744073709551647u128 as u64);
}
}
// check: ARITHMETIC_ERROR

//! new-transaction
script {
fun main() {
    (340282366920938463463374607431768211455u128 as u64);
}
}
// check: ARITHMETIC_ERROR
