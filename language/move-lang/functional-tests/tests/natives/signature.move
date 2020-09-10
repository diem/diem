// Test fot public key validation

script {
use 0x1::Signature;

fun main() {

    // from RFC 8032
    let valid_pubkey =  x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    let short_pubkey = x"100";
    // concatenation of the two above
    let long_pubkey = x"1003d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    let invalid_pubkey = x"0000000000000000000000000000000000000000000000000000000000000000";

    assert(Signature::ed25519_validate_pubkey(copy valid_pubkey), 9003);
    assert(!Signature::ed25519_validate_pubkey(copy short_pubkey), 9004);
    assert(!Signature::ed25519_validate_pubkey(copy long_pubkey), 9005);
    assert(!Signature::ed25519_validate_pubkey(copy invalid_pubkey), 9006);

    let short_signature = x"100";
    let long_signature = x"0062d6be393b8ec77fb2c12ff44ca8b5bd8bba83b805171bc99f0af3bdc619b20b8bd529452fe62dac022c80752af2af02fb610c20f01fb67a4d72789db2b8b703";
    let valid_signature = x"62d6be393b8ec77fb2c12ff44ca8b5bd8bba83b805171bc99f0af3bdc619b20b8bd529452fe62dac022c80752af2af02fb610c20f01fb67a4d72789db2b8b703";

    // now check that Signature::ed25519_verify works with well- and ill-formed data and never aborts
    // valid signature, invalid pubkey (too short, too long, bad small subgroup)
    assert(!Signature::ed25519_verify(copy valid_signature, copy short_pubkey, x""), 9004);
    assert(!Signature::ed25519_verify(copy valid_signature, copy long_pubkey, x""), 9005);
    assert(!Signature::ed25519_verify(copy valid_signature, copy invalid_pubkey, x""), 9006);
    // invalid signature, valid pubkey
    assert(!Signature::ed25519_verify(copy short_signature, copy valid_pubkey, x""), 9007);
    assert(!Signature::ed25519_verify(copy long_signature, copy valid_pubkey, x""), 9008);

    // valid (lengthwise) signature, valid pubkey, but signature doesn't match message
    assert(!Signature::ed25519_verify(copy valid_signature, copy valid_pubkey, x""), 9009);

    // all three valid
    let message = x"0000000000000000000000000000000000000000000000000000000000000000";
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    assert(Signature::ed25519_verify(valid_signature, pubkey, message), 9010);
}
}
