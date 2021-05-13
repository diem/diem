script {
use DiemFramework::XUS;

fun main(account: signer) {
    let account = &account;
    // This will fail because XUS has already been initialized
    XUS::initialize(account,  account);
}
}
// check: Keep(ABORTED { code: 1, location: 00000000000000000000000000000001::DiemTimestamp })
