//! account: parent, 10000000, 0, vasp
//! account: bob, 10000000, 0, unhosted

//! new-transaction
script {
use 0x0::VASPRegistry;
use 0x0::Vector;
use 0x0::Transaction;
fun main() {
    let registered_vasps = VASPRegistry::registered_vasp_addresses();
    Transaction::assert(!Vector::contains(&registered_vasps, &0xA), 0);
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    LibraAccount::create_parent_vasp_account<LBR::T>(
        0xA, copy dummy_auth_key_prefix, x"A1", x"A2", copy pubkey
    );
}
}
// check: EXECUTED

//! new-transaction
script {
use 0x0::VASPRegistry;
use 0x0::Vector;
use 0x0::Transaction;
fun main() {
    let registered_vasps = VASPRegistry::registered_vasp_addresses();
    Transaction::assert(Vector::contains(&registered_vasps, &0xA), 1);
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::VASPRegistry;
use 0x0::Vector;
use 0x0::Transaction;
use 0x0::VASP;
fun main() {
    VASP::delist_vasp(0xA);
    let registered_vasps = VASPRegistry::registered_vasp_addresses();
    Transaction::assert(!Vector::contains(&registered_vasps, &0xA), 2);
}
}
// check: EXECUTED

//! new-transaction
script {
use 0x0::VASP;
fun main() {
    VASP::delist_vasp(0xA);
}
}
// check: ABORTED
// check: 1002
