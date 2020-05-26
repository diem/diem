// Check that the add_all_currencies flag for parent and child VASP account creation works

//! sender: association
script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
fun main() {
    // create a parent VASP that accepts one currency
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    let add_all_currencies = false;
    LibraAccount::create_parent_vasp_account<LBR::T>(
        0xA, copy dummy_auth_key_prefix, x"A1", x"A2", copy pubkey, add_all_currencies
    );
    Transaction::assert(LibraAccount::accepts_currency<LBR::T>(0xA), 2001);
    Transaction::assert(!LibraAccount::accepts_currency<Coin1::T>(0xA), 2002);
    Transaction::assert(!LibraAccount::accepts_currency<Coin2::T>(0xA), 2003);

    // now create a parent VASP that accepts all currencies
    add_all_currencies = true;
    LibraAccount::create_parent_vasp_account<LBR::T>(
        0xB, dummy_auth_key_prefix, x"A1", x"A2", pubkey, add_all_currencies
    );
    Transaction::assert(LibraAccount::accepts_currency<LBR::T>(0xB), 2004);
    Transaction::assert(LibraAccount::accepts_currency<Coin1::T>(0xB), 2005);
    Transaction::assert(LibraAccount::accepts_currency<Coin2::T>(0xB), 2006);
}
}
// check: EXECUTED

//! account: parent, 10000000, 0, vasp
//! new-transaction
//! sender: parent
script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
fun main(parent_vasp: &signer) {
    // create a child VASP that accepts one currency
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    let add_all_currencies = false;
    LibraAccount::create_child_vasp_account<LBR::T>(
        parent_vasp, 0xAA, copy dummy_auth_key_prefix, add_all_currencies
    );

    Transaction::assert(LibraAccount::accepts_currency<LBR::T>(0xAA), 2007);
    Transaction::assert(!LibraAccount::accepts_currency<Coin1::T>(0xAA), 2008);
    Transaction::assert(!LibraAccount::accepts_currency<Coin2::T>(0xAA), 2009);

    // now create a child VASP that accepts all currencies
    add_all_currencies = true;
    LibraAccount::create_child_vasp_account<LBR::T>(
        parent_vasp, 0xBB, dummy_auth_key_prefix, add_all_currencies
    );
    Transaction::assert(LibraAccount::accepts_currency<LBR::T>(0xBB), 2010);
    Transaction::assert(LibraAccount::accepts_currency<Coin1::T>(0xBB), 2011);
    Transaction::assert(LibraAccount::accepts_currency<Coin2::T>(0xBB), 2012);
}
}
// check: EXECUTED
