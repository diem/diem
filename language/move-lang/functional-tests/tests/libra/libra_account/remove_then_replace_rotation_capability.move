script {
use 0x1::LibraAccount;
use 0x1::Signer;
fun main(account: &signer) {
    let sender = Signer::address_of(account);
    let old_auth_key = LibraAccount::authentication_key(sender);

    // by default, an account has not delegated its key rotation capability
    assert(!LibraAccount::delegated_key_rotation_capability(sender), 50);

    // extracting the capability should flip the flag
    let cap = LibraAccount::extract_key_rotation_capability(account);
    assert(LibraAccount::delegated_key_rotation_capability(sender), 51);

    // and the sender should be able to rotate
    LibraAccount::rotate_authentication_key(&cap, old_auth_key);

    // restoring the capability should flip the flag back
    LibraAccount::restore_key_rotation_capability(cap);
    assert(!LibraAccount::delegated_key_rotation_capability(sender), 52);
}
}
// check: "Keep(EXECUTED)"

// Extracting the capability should preclude rotation
//! new-transaction
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    let cap = LibraAccount::extract_key_rotation_capability(account);
    let cap2 = LibraAccount::extract_key_rotation_capability(account);

    // should fail
    LibraAccount::rotate_authentication_key(&cap2, x"00");
    LibraAccount::restore_key_rotation_capability(cap);
    LibraAccount::restore_key_rotation_capability(cap2);
}
}
// check: "Keep(ABORTED { code: 2305,"
// check: location: ::LibraAccount
