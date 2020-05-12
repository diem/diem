// Test that rotating the key used to pre-approve payments works

//! account: alice
//! account: bob
//! account: charlie

// setup: alice publishes an approved payment resource, then rotates the key

//! sender: alice
script {
use 0x0::ApprovedPayment;
fun main() {
    let pubkey = x"aa306695ca5ade60240c67b9b886fe240a6f009b03e43e45838334eddeae49fe";
    ApprovedPayment::publish(pubkey);
    ApprovedPayment::rotate_sender_key(x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d");
}
}
// check: EXECUTED

// offline: alice generates payment id 0, signs it, and sends ID + signature to bob
// online: now bob puts the payment id and signature in transaction and uses it to pay Alice

//! new-transaction
//! sender: bob
script {
use 0x0::ApprovedPayment;
use 0x0::LBR;
fun main() {
    let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
    let signature = x"62d6be393b8ec77fb2c12ff44ca8b5bd8bba83b805171bc99f0af3bdc619b20b8bd529452fe62dac022c80752af2af02fb610c20f01fb67a4d72789db2b8b703";
    ApprovedPayment::deposit_to_payee<LBR::T>({{alice}}, 1000, payment_id, signature);
}
}
// check: EXECUTED

// charlie publishes an approved payment resource, then tries to rotate to an invalid key

//! new-transaction
//! sender: charlie
script {
use 0x0::ApprovedPayment;
fun main() {
    let pubkey = x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    ApprovedPayment::publish(pubkey);
    // rotate to an invalid key
    ApprovedPayment::rotate_sender_key(x"0000000000000000000000000000000000000000000000000000000000000000");
}
}
// check: ABORTED
// check: 9003
