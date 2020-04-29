// Test the end-to-end approved payment flow by (1) pre-approving a payment to alice from bob with
// a valid signature from alice (should work) and (2) the same, but with an invalid signature
// (shouldn't work).

//! account: alice
//! account: bob

// setup: alice publishes an approved payment resource

//! sender: alice
script {
use 0x0::ApprovedPayment;
fun main() {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    ApprovedPayment::publish(pubkey)
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

// same as above, but with an invalid signature. should now abort

//! new-transaction
//! sender: bob
script {
use 0x0::ApprovedPayment;
use 0x0::LBR;
fun main() {
    let payment_id = x"7";
    let signature = x"62d6be393b8ec77fb2c12ff44ca8b5bd8bba83b805171bc99f0af3bdc619b20b8bd529452fe62dac022c80752af2af02fb610c20f01fb67a4d72789db2b8b703";
    ApprovedPayment::deposit_to_payee<LBR::T>({{alice}}, 1000, payment_id, signature);
}
}

// check: ABORTED
// check: 9002
