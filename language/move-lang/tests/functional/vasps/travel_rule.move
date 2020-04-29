// Register a sender and recipient VASP, check that applicable payments go through the travel rule
// amount and signature checks

//! account: payer, 10000000, 0, true
//! account: child, 10000000, 0, true
//! account: payee, 10000000, 0, true
//! account: alice
//! account: bob

// payer applies to be a VASP
//! sender: payer
script {
use 0x0::VASP;
fun main() {
    let pubkey = x"0000000d7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    VASP::apply_for_vasp_root_credential(x"AAA", x"BBB", x"CCC", pubkey);
}
}
// check: EXECUTED

// payee applies to be a VASP
//! new-transaction
//! sender: payee
script {
use 0x0::VASP;
fun main() {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    VASP::apply_for_vasp_root_credential(x"DDD", x"EEE", x"FFF", pubkey);
}
}
// check: EXECUTED

// Association approves both
//! new-transaction
//! sender: association
script {
use 0x0::Association;
use 0x0::VASP;
fun main() {
    Association::apply_for_privilege<VASP::CreationPrivilege>();
    Association::grant_privilege<VASP::CreationPrivilege>({{association}});
    VASP::grant_vasp({{payer}});
    VASP::grant_vasp({{payee}});
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
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
    let signature = x"62d6be393b8ec77fb2c12ff44ca8b5bd8bba83b805171bc99f0af3bdc619b20b8bd529452fe62dac022c80752af2af02fb610c20f01fb67a4d72789db2b8b703";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payee}}, 1000, payment_id, signature);
}
}
// check: EXECUTED

// transaction >= 1000 threshold goes through signature verification with invalid signature, aborts
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"1000000000000000000000000000000000000000000000000000000000000001";
    let signature = x"62d6be393b8ec77fb2c12ff44ca8b5bd8bba83b805171bc99f0af3bdc619b20b8bd529452fe62dac022c80752af2af02fb610c20f01fb67a4d72789db2b8b703";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payee}}, 1000, payment_id, signature);
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
    let signature = x"62d6be393b8ec77fb2c12ff44ca8b5bd8bba83b805171bc99f0af3bdc619b20b8bd529452fe62dac022c80752af2af02fb610c20f01fb67a4d72789db2b8b703";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{payee}}, 1000, payment_id, signature);
}
}
// check: ABORTED
// check: 9002

// transaction < 1000 threshold not subject to travel rule, goes through with any signature
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

// Test that intra-VASP transactions are not subject to the travel rule.
// First, we must do some setup by creating a child VASP for payer

// payer allows child accounts
//! new-transaction
//! sender: payer
script {
use 0x0::VASP;
fun main() {
    VASP::allow_child_accounts();
}
}
// check: EXECUTED

// apply to be child of payer
//! new-transaction
//! sender: child
script {
use 0x0::VASP;
fun main() {
    VASP::apply_for_child_vasp_credential({{payer}});
}
}
// check: EXECUTED

// payer allows child accounts
//! new-transaction
//! sender: payer
script {
use 0x0::VASP;
fun main() {
    VASP::grant_child_account({{child}});
}
}
// check: EXECUTED

// intra-VASP transaction >= 1000 threshold, should go throug with any signature
//! new-transaction
//! sender: payer
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let payment_id = x"0000000000000000000000000000000000000000000000000000000000000000";
    let signature = x"";

    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{child}}, 1001, payment_id, signature);
}
}
// check: EXECUTED

// same thing, but from child -> parent
//! new-transaction
//! sender: child
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


// check that unhosted wallet <-> VASP transactions are not subject to the travel rule

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

// finally, check that unhosted <-> unhosted transactions are not subject to the travel rule

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
