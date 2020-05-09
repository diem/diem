// Register a sender and recipient VASP, check that applicable payments go through dual attestation
// checks

//! account: payer, 10000000, 0, vasp
//! account: payee, 10000000, 0, vasp
//! account: alice, 10000000, 0, unhosted
//! account: bob,   10000000, 0, unhosted

// Association sets compliance keys for bith payer and payee
//! new-transaction
//! sender: association
script {
use 0x0::LibraAccount;
fun main(account: &signer) {
    LibraAccount::add_parent_vasp_role_from_association(
        account, {{payer}}, x"A", x"B", x"0000000d7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d"
    );
    LibraAccount::add_parent_vasp_role_from_association(
        account, {{payee}}, x"C", x"D", x"e14c7ddb7713c9c7315f337393f4261d1713a5f8c6c6e14c4986e616460251e6"
    );
}
}
// check: EXECUTED

// done with setup; check inter-VASP properties:
// (1) transactions >= 1000 need a valid signature
// (2) transactions < 1000 do not

// transaction >= 1000 threshold goes through signature verification with valid signature, passes
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LCS;
use 0x0::LibraAccount;
use 0x0::Transaction;
fun main() {
    let sender_address = 0xa4a46d1b1421502568a4a6ac326d7250;
    // if the sender address changes, signature below will need to updated
    Transaction::assert(Transaction::sender() == sender_address, 999);
    let ref_id = LCS::to_bytes(&7777);
    // the sad scheme to regenerate this signature:
    // (1) Go to language/e2e_tests/src/tests/transaction_builder, "dual_attestation_payment"
    // (2) replace the Rust `payment_sender.address()` with
    //     libra_types::account_address::AccountAddress::from_hex_literal(<literal bound to sender_address above>)
    // (3) Add a debug print to grab the new signature, paste it here
    let signature = x"4e597148bdb68de6a5f32e1ccd7e4f27044724b69f99328fdfb3c14c63d37d6c63d95a9a549a59ad30320fbd6ce375fbfc275e941e133bf90d4064fdf8aca903";
    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payee}}, 1000000, ref_id, signature);
}
}
// check: EXECUTED


// transaction >= 1000 threshold goes through signature verification with
// invalid signature, fails passes
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
    let signature = x"ab";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payee}}, 1000, payment_id, signature);
}
}
// check: ABORTED
// check: 9001

// transaction >= 1000 threshold goes through signature verification with invalid signature, aborts
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LCS;
use 0x0::LibraAccount;
fun main() {
    let ref_id = LCS::to_bytes(&9999);
    let signature = x"8d83d481068a7b73a914e7d53f84cbfb8d55cdbd2306e17dae14fa0e6fa8ab0cea9f4c1bab22b701e57aa2e0e3f15bb6b40d62a32b7e158a384b6529f6463a09";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payee}}, 1000, ref_id, signature);
}
}
// check: ABORTED
// check: 9002

// similar, but with empty payment ID (make sure signature is still invalid!)
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"";
    let signature = x"8d83d481068a7b73a914e7d53f84cbfb8d55cdbd2306e17dae14fa0e6fa8ab0cea9f4c1bab22b701e57aa2e0e3f15bb6b40d62a32b7e158a384b6529f6463a09";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payee}}, 1000, payment_id, signature);
}
}
// check: ABORTED
// check: 9002

// transaction < 1000 threshold not subject to dual attestation, goes through with any signature
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
    let signature = x"";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payee}}, 999, payment_id, signature);
}
}
// check: EXECUTED

// Test that intra-VASP transactions are not subject to dual attestation
// First, we must do some setup by creating a child VASP for payer

// create a child VASP at AA
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main(parent_vasp: &signer) {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    LibraAccount::create_child_vasp_account<LBR::T>(0xAA, dummy_auth_key_prefix, parent_vasp);
}
}
// check: EXECUTED

// intra-VASP transaction >= 1000 threshold, should go through with any signature
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
    let signature = x"";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>(0xAA, 1001, payment_id, signature);
}
}
// check: EXECUTED

// TODO: add E2E test feature to facilitate sending from child VASP
// // same thing, but from child -> parent
// //! new-transaction
// //! sender: 0xAA
// script {
// use 0x0::LBR;
// use 0x0::LibraAccount;
// fun main() {
//     let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
//     let signature = x"";

//     LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payer}}, 1001, payment_id, signature);
// }
// }
// // chec: EXECUTED


// check that unhosted wallet <-> VASP transactions do not require dual attestation

// VASP -> wallet direction
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
    let signature = x"";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{alice}}, 1001, payment_id, signature);
}
}
// check: EXECUTED

// wallet -> VASP direction
//! new-transaction
//! sender: alice
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
    let signature = x"";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payer}}, 1001, payment_id, signature);
}
}
// check: EXECUTED

// finally, check that unhosted <-> unhosted transactions do not require dual attestation

// wallet -> VASP direction
//! new-transaction
//! sender: alice
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
    let signature = x"";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{bob}}, 1001, payment_id, signature);
}
}
// check: EXECUTED

//! new-transaction
//! sender: payer
script {
use 0x0::Transaction;
use 0x0::LibraAccount;
// rotate the compliance public key
fun main(account: &signer) {
    let old_pubkey = LibraAccount::compliance_public_key({{payer}});
    let new_pubkey = x"1111110d7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    Transaction::assert(&old_pubkey != &new_pubkey, 777);
    LibraAccount::rotate_compliance_public_key(account, copy new_pubkey);
    Transaction::assert(LibraAccount::compliance_public_key({{payer}}) == new_pubkey, 777);
}
}
// check: EXECUTED

//! new-transaction
//! sender: payer
script {
use 0x0::LibraAccount;
// Try to rotate to a (structurally) malformed compliance public key
fun main(account: &signer) {
    let new_pubkey = x"d7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    LibraAccount::rotate_compliance_public_key(account, copy new_pubkey);
}
}
// check: ABORTED
// check: 7004
