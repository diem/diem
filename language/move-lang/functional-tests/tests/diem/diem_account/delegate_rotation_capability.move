//! account: alice
//! account: bob

//! sender: alice
// A module that lets the account owner rotate their auth key as they normally would, but also
// allows a different address `master_key_address` to rotate the auth key. This is useful for
// implementing (e.g.) a hot wallet with a cold recovery key.
module SharedKeyRotation {
    use 0x1::DiemAccount;
    use 0x1::Signer;

    struct T has key {
        // cap.address can rotate the auth key for cap.address
        cap: DiemAccount::KeyRotationCapability,
        // master_key_address can also rotate the auth key for cap.address
        master_key_address: address,
    }

    // Publish a SharedRotation resource for the account `cap.address` with master key
    // `master_key_address` under the sender's account
    public fun publish(account: &signer, cap: DiemAccount::KeyRotationCapability, master_key_address: address) {
        move_to(account, T { cap, master_key_address });
    }

    // Rotate the auth key for the account at wallet_address/SharedKeyRotation.SharedKeyRotation/cap/address to
    // new_key
    public fun rotate(account: &signer, wallet_address: address, new_key: vector<u8>) acquires T {
        let wallet_ref = borrow_global_mut<T>(wallet_address);
        let sender = Signer::address_of(account);
        let cap_addr = *DiemAccount::key_rotation_capability_address(&wallet_ref.cap);
        assert((wallet_ref.master_key_address == sender) || (cap_addr == sender), 77);
        DiemAccount::rotate_authentication_key(&wallet_ref.cap, new_key);
    }
}

//! new-transaction
//! sender: alice
script {
use {{alice}}::SharedKeyRotation;
use 0x1::DiemAccount;
// create a SharedKeyRotation for Alice's account with Bob's account key as the master key
fun main(account: signer) {
    let account = &account;
    assert(DiemAccount::sequence_number({{alice}}) == 1, 77);
    SharedKeyRotation::publish(account, DiemAccount::extract_key_rotation_capability(account), {{bob}});
}
}

//! new-transaction
//! sender: alice
script {
use {{alice}}::SharedKeyRotation;
use 0x1::DiemAccount;
// Alice can rotate her key. Here, she rotates it to its original value
fun main(account: signer) {
    let account = &account;
    assert(DiemAccount::sequence_number({{alice}}) == 2, 78);
    SharedKeyRotation::rotate(
        account,
        {{alice}},
        x"08a88082abf9fbb576bdb6969143ee6384066e363c48e041c8da1e08b9fc931f",
    );
}
}

//! new-transaction
//! sender: bob
script {
use {{alice}}::SharedKeyRotation;
use 0x1::DiemAccount;
// Bob can too. Here, he zeroes it out to stop Alice from sending any transactions
fun main(account: signer) {
    let account = &account;
    assert(DiemAccount::sequence_number({{alice}}) == 3, 78);
    SharedKeyRotation::rotate(
        account,
        {{alice}},
        x"0000000000000000000000000000000000000000000000000000000000000000",
    );
}
}

//! new-transaction
//! sender: alice
// Alice should no longer be able to send a tx from her account
script {
fun main() {
}
}
// check: Discard(INVALID_AUTH_KEY)

//! new-transaction
//! sender: bob
script {
use {{alice}}::SharedKeyRotation;
use 0x1::BCS;
use 0x1::DiemAccount;
use 0x1::Vector;
// Bob now rotates the key back to its old value
fun main(account: signer) {
    let account = &account;
    assert(DiemAccount::sequence_number({{alice}}) == 3, 78);
    // simulates how an auth_key is created
    // details to be found in DiemAccount::make_account
    let newkey = Vector::empty();
    Vector::append(&mut newkey, {{alice::auth_key}});
    Vector::append(&mut newkey, BCS::to_bytes(&{{alice}}));
    SharedKeyRotation::rotate(account, {{alice}}, newkey);
}
}

//! new-transaction
//! sender: alice
// And then Alice should be able to send a tx once again
script {
use 0x1::DiemAccount;
fun main() {
    assert(DiemAccount::sequence_number({{alice}}) == 3, 79);
}
}
