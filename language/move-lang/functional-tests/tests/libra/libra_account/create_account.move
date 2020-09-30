//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
use 0x1::LCS;
fun main(account: &signer) {
    let addr: address = 0x111101;
    assert(!LibraAccount::exists_at(addr), 83);
    LibraAccount::create_parent_vasp_account<Coin1>(account, addr, LCS::to_bytes(&addr), x"aa", false);
}
}

//! new-transaction
script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let addr: address = 0x111101;
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, addr, 10, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
    assert(LibraAccount::balance<Coin1>(addr) == 10, 84);
    assert(LibraAccount::sequence_number(addr) == 0, 84);
}
}
