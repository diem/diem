//! account: parent, 10000000, 0, vasp
//! account: bob, 10000000, 0, unhosted

// create a parent VASP
//! new-transaction
//! sender: association
script {
use 0x1::LBR::LBR;
use 0x1::LibraAccount;
use 0x1::LibraTimestamp;
use 0x1::VASP;
fun main(assoc: &signer) {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    let add_all_currencies = false;
    LibraAccount::create_parent_vasp_account<LBR>(
        assoc, 0xA, copy dummy_auth_key_prefix, x"A1", x"A2", copy pubkey, add_all_currencies
    );

    assert(VASP::is_vasp(0xA), 2001);
    assert(VASP::is_parent(0xA), 2002);
    assert(!VASP::is_child(0xA), 2003);

    assert(VASP::parent_address(0xA) == 0xA, 2005);
    assert(VASP::compliance_public_key(0xA) == copy pubkey, 2006);
    assert(VASP::human_name(0xA) == x"A1", 2007);
    assert(VASP::base_url(0xA) == x"A2", 2008);
    assert(
        VASP::expiration_date(0xA) > LibraTimestamp::now_microseconds(),
        2009
    );

    // set up parent account as a VASP
    // TODO: remove this once //! account works
    LibraAccount::add_parent_vasp_role_from_association(
        assoc, {{parent}}, x"A1", x"A2", pubkey,
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
use 0x1::VASP;
fun main(parent_vasp: &signer) {
    let old_pubkey = VASP::compliance_public_key({{parent}});
    let new_pubkey = x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    assert(old_pubkey != copy new_pubkey, 2011);
    VASP::rotate_compliance_public_key(parent_vasp, copy new_pubkey);
    assert(VASP::compliance_public_key({{parent}}) == new_pubkey, 2015);
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
// check: ABORTED
// check: 88
