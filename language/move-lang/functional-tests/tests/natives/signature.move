// Test fot public key validation

script {
use 0x0::Signature;
use 0x0::Transaction;

fun main() {

    // from RFC 8032
    let valid_pubkey =  x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    let short_pubkey = x"100";
    // concatenation of the two above
    let long_pubkey = x"1003d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    let invalid_pubkey = x"0000000000000000000000000000000000000000000000000000000000000000";

    Transaction::assert(Signature::ed25519_validate_pubkey(valid_pubkey), 9003);
    Transaction::assert(!Signature::ed25519_validate_pubkey(short_pubkey), 9003);
    Transaction::assert(!Signature::ed25519_validate_pubkey(long_pubkey), 9003);
    Transaction::assert(!Signature::ed25519_validate_pubkey(invalid_pubkey), 9003);
}
}
