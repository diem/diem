script {
use 0x1::Coin1;

fun main(account: &signer) {
    // This will fail because Coin1 has already been initialized
    Coin1::initialize(account,  account);
}
}
// check: Keep(ABORTED { code: 1, location: 00000000000000000000000000000001::LibraTimestamp })
