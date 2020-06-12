// Check that the add_all_currencies flag for parent and child VASP account creation works

//! sender: association
script {
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::LBR::LBR;
use 0x0::LibraAccount;
fun main(assoc: &signer) {
    // create a parent VASP that accepts one currency
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    let add_all_currencies = false;
    LibraAccount::create_parent_vasp_account<LBR>(
        assoc, 0xA, copy dummy_auth_key_prefix, x"A1", x"A2", copy pubkey, add_all_currencies
    );
    assert(LibraAccount::accepts_currency<LBR>(0xA), 2001);
    assert(!LibraAccount::accepts_currency<Coin1>(0xA), 2002);
    assert(!LibraAccount::accepts_currency<Coin2>(0xA), 2003);

    // now create a parent VASP that accepts all currencies
    add_all_currencies = true;
    LibraAccount::create_parent_vasp_account<LBR>(
        assoc, 0xB, dummy_auth_key_prefix, x"A1", x"A2", copy pubkey, add_all_currencies
    );
    assert(LibraAccount::accepts_currency<LBR>(0xB), 2004);
    assert(LibraAccount::accepts_currency<Coin1>(0xB), 2005);
    assert(LibraAccount::accepts_currency<Coin2>(0xB), 2006);

    // set up parent account as a VASP
    // TODO: remove this once //! account works
    LibraAccount::add_parent_vasp_role_from_association(
        assoc, {{parent}}, x"A1", x"A2", pubkey,
    );
}
}
// check: EXECUTED

//! account: parent, 10000000, 0, vasp
//! new-transaction
//! sender: parent
script {
use 0x0::Coin1::Coin1;
use 0x0::Coin2::Coin2;
use 0x0::LBR::LBR;
use 0x0::LibraAccount;
fun main(parent_vasp: &signer) {
    // create a child VASP that accepts one currency
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    let add_all_currencies = false;
    LibraAccount::create_child_vasp_account<LBR>(
        parent_vasp, 0xAA, copy dummy_auth_key_prefix, add_all_currencies
    );

    assert(LibraAccount::accepts_currency<LBR>(0xAA), 2007);
    assert(!LibraAccount::accepts_currency<Coin1>(0xAA), 2008);
    assert(!LibraAccount::accepts_currency<Coin2>(0xAA), 2009);

    // now create a child VASP that accepts all currencies
    add_all_currencies = true;
    LibraAccount::create_child_vasp_account<LBR>(
        parent_vasp, 0xBB, dummy_auth_key_prefix, add_all_currencies
    );
    assert(LibraAccount::accepts_currency<LBR>(0xBB), 2010);
    assert(LibraAccount::accepts_currency<Coin1>(0xBB), 2011);
    assert(LibraAccount::accepts_currency<Coin2>(0xBB), 2012);
}
}
// check: EXECUTED
