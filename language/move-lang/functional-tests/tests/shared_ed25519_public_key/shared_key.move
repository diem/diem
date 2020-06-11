// Test that publishing and rotating the shared key works as expected.
// It is tricky to test that a transaction can still be sent after key rotation in a functional
// test, so that case is covered in an e2e test.

//! account: alice

script {
use 0x1::LibraAccount;
use 0x1::SharedEd25519PublicKey;
fun main(account: &signer) {
    let old_auth_key = LibraAccount::authentication_key({{default}});
    // from RFC 8032
    let pubkey1 = x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    SharedEd25519PublicKey::publish(account, copy pubkey1);
    let new_auth_key = LibraAccount::authentication_key({{default}});

    // check that publishing worked
    assert(SharedEd25519PublicKey::exists_at({{default}}), 3000);
    assert(SharedEd25519PublicKey::key({{default}}) == pubkey1, 3001);

    // publishing should extract the sender's key rotation capability
    assert(LibraAccount::delegated_key_rotation_capability({{default}}), 3002);
    // make sure the sender's auth key has changed
    assert(copy new_auth_key != old_auth_key, 3003);

    // now rotate to another pubkey and redo the key-related checks
    // from RFC 8032
    let pubkey2 = x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    SharedEd25519PublicKey::rotate_key(account, copy pubkey2);
    assert(SharedEd25519PublicKey::key({{default}}) == pubkey2, 3004);
    // make sure the auth key changed again
    assert(new_auth_key != LibraAccount::authentication_key({{default}}), 3005);
}
}
// check: EXECUTED

// publishing a key with a bad length should fail
//! new-transaction
//! sender: alice
script {
use 0x1::SharedEd25519PublicKey;
fun main(account: &signer) {
    let invalid_pubkey = x"000";
    SharedEd25519PublicKey::publish(account, invalid_pubkey)
}
}
// check: ABORTED
// check: 9003

// publishing a key with a bad length should fail
//! new-transaction
//! sender: alice
script {
use 0x1::SharedEd25519PublicKey;
fun main(account: &signer) {
    let invalid_pubkey = x"10003d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    SharedEd25519PublicKey::publish(account, invalid_pubkey)
}
}
// check: ABORTED
// check: 9003


// rotating to a key with a bad length should fail
//! new-transaction
//! sender: alice
script {
use 0x1::SharedEd25519PublicKey;
fun main(account: &signer) {
    // from RFC 8032
    let valid_pubkey =  x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    SharedEd25519PublicKey::publish(account, valid_pubkey);
    // now rotate to an invalid key
    let invalid_pubkey = x"10000";
    SharedEd25519PublicKey::rotate_key(account, invalid_pubkey)
}
}
// check: ABORTED
// check: 9003

// rotating to a key with a good length but bad contents should fail
//! new-transaction
//! sender: alice
script {
use 0x1::SharedEd25519PublicKey;
fun main(account: &signer) {
    let valid_pubkey =  x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    SharedEd25519PublicKey::publish(account, valid_pubkey);
    // now rotate to an invalid key
    let invalid_pubkey = x"0000000000000000000000000000000000000000000000000000000000000000";
    SharedEd25519PublicKey::rotate_key(account, invalid_pubkey)
}
}
// check: ABORTED
// check: 9003
