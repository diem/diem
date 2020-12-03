script {
use 0x1::XUS;

fun main(account: &signer) {
    // This will fail because XUS has already been initialized
    XUS::initialize(account,  account);
}
}
// check: Keep(ABORTED { code: 1, location: 00000000000000000000000000000001::DiemTimestamp })
