//! account: parent, 0, 0, address
//! account: bob, 10000000

// create a parent VASP
//! new-transaction
//! sender: libraroot
script {
use 0x1::DualAttestation;
use 0x1::LBR::LBR;
use 0x1::LibraAccount;
use 0x1::LibraTimestamp;
use 0x1::VASP;
fun main(lr_account: &signer) {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    let add_all_currencies = false;

    LibraAccount::create_parent_vasp_account<LBR>(
        lr_account,
        {{parent}},
        {{parent::auth_key}},
        x"A1",
        x"A2",
        copy pubkey,
        add_all_currencies,
    );

    assert(VASP::is_vasp({{parent}}), 2001);
    assert(VASP::is_parent({{parent}}), 2002);
    assert(!VASP::is_child({{parent}}), 2003);

    assert(VASP::parent_address({{parent}}) == {{parent}}, 2005);
    assert(DualAttestation::compliance_public_key({{parent}}) == copy pubkey, 2006);
    assert(DualAttestation::human_name({{parent}}) == x"A1", 2007);
    assert(DualAttestation::base_url({{parent}}) == x"A2", 2008);
    assert(
        DualAttestation::expiration_date({{parent}}) > LibraTimestamp::now_microseconds(),
        2009
    );

}
}
// check: EXECUTED

// create some child VASP accounts
//! new-transaction
//! sender: parent
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
use 0x1::VASP;
fun main(parent_vasp: &signer) {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    let add_all_currencies = false;
    assert(VASP::num_children({{parent}}) == 0, 2010);
    LibraAccount::create_child_vasp_account<LBR>(
        parent_vasp, 0xAA, copy dummy_auth_key_prefix, add_all_currencies
    );
    assert(VASP::num_children({{parent}}) == 1, 2011);
    assert(VASP::parent_address(0xAA) == {{parent}}, 2012);
    LibraAccount::create_child_vasp_account<LBR>(
        parent_vasp, 0xBB, dummy_auth_key_prefix, add_all_currencies
    );
    assert(VASP::num_children({{parent}}) == 2, 2013);
    assert(VASP::parent_address(0xBB) == {{parent}}, 2014);
}
}
// check: EXECUTED

//! new-transaction
//! sender: parent
script {
use 0x1::DualAttestation;
fun main(parent_vasp: &signer) {
    let old_pubkey = DualAttestation::compliance_public_key({{parent}});
    let new_pubkey = x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    assert(old_pubkey != copy new_pubkey, 2011);
    DualAttestation::rotate_compliance_public_key(parent_vasp, copy new_pubkey);
    assert(DualAttestation::compliance_public_key({{parent}}) == new_pubkey, 2015);
}
}
// check: EXECUTED

// getting the parent VASP address of a non-VASP should abort
//! new-transaction
//! sender: bob
script {
use 0x1::VASP;
fun main() {
    assert(VASP::parent_address({{bob}}) == {{parent}}, 2016);
}
}
// check: "Keep(ABORTED { code: 775,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::VASP;
fun main(account: &signer) {
    VASP::initialize(account)
}
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
script {
use 0x1::VASP;
use 0x1::LibraTimestamp;
fun main(account: &signer) {
    LibraTimestamp::reset_time_has_started_for_test();
    VASP::initialize(account);
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::VASP;
fun main(account: &signer) {
    VASP::publish_parent_vasp_credential(account, account);
    abort 99
}
}
// check: "Keep(ABORTED { code: 771,"

//! new-transaction
//! sender: blessed
script {
use 0x1::VASP;
fun main(account: &signer) {
    VASP::publish_parent_vasp_credential(account, account);
}
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: blessed
script {
use 0x1::VASP;
fun main(account: &signer) {
    VASP::publish_child_vasp_credential(account, account);
}
}
// check: "Keep(ABORTED { code: 771,"

//! new-transaction
//! sender: blessed
script {
use 0x1::VASP;
fun main(account: &signer) {
    VASP::publish_child_vasp_credential(account, account);
}
}
// check: "Keep(ABORTED { code: 771,"

//! new-transaction
//! sender: parent
script {
use 0x1::VASP;
fun main(account: &signer) {
    VASP::publish_child_vasp_credential(account, account);
}
}
// check: "Keep(ABORTED { code: 262,"

//! new-transaction
//! sender: parent
script {
use 0x1::VASP;
fun main() {
    assert(!VASP::is_same_vasp({{parent}}, {{blessed}}), 42);
}
}
// check: EXECUTED

//! new-transaction
//! sender: parent
script {
use 0x1::VASP;
fun main() {
    assert(!VASP::is_same_vasp({{blessed}}, {{parent}}), 42);
}
}
// check: EXECUTED
