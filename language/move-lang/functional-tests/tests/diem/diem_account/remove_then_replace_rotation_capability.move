script {
use 0x1::DiemAccount;
use 0x1::Signer;
fun main(account: signer) {
    let account = &account;
    let sender = Signer::address_of(account);
    let old_auth_key = DiemAccount::authentication_key(sender);

    // by default, an account has not delegated its key rotation capability
    assert(!DiemAccount::delegated_key_rotation_capability(sender), 50);

    // extracting the capability should flip the flag
    let cap = DiemAccount::extract_key_rotation_capability(account);
    assert(DiemAccount::delegated_key_rotation_capability(sender), 51);

    // and the sender should be able to rotate
    DiemAccount::rotate_authentication_key(&cap, old_auth_key);

    // restoring the capability should flip the flag back
    DiemAccount::restore_key_rotation_capability(cap);
    assert(!DiemAccount::delegated_key_rotation_capability(sender), 52);
}
}
// check: "Keep(EXECUTED)"

// Extracting the capability should preclude rotation
//! new-transaction
script {
use 0x1::DiemAccount;
fun main(account: signer) {
    let account = &account;
    let cap = DiemAccount::extract_key_rotation_capability(account);
    let cap2 = DiemAccount::extract_key_rotation_capability(account);

    // should fail
    DiemAccount::rotate_authentication_key(&cap2, x"00");
    DiemAccount::restore_key_rotation_capability(cap);
    DiemAccount::restore_key_rotation_capability(cap2);
}
}
// check: "Keep(ABORTED { code: 2305,"
// check: location: ::DiemAccount
