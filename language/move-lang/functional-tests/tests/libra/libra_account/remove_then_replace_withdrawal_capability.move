script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
use 0x1::Signer;
fun main(account: &signer) {
    let sender = Signer::address_of(account);

    // by default, an account has not delegated its withdrawal capability
    assert(!LibraAccount::delegated_withdraw_capability(sender), 50);

    // make sure we report that the capability has been extracted
    let cap = LibraAccount::extract_withdraw_capability(account);
    assert(LibraAccount::delegated_withdraw_capability(sender), 51);

    // and the sender should be able to withdraw with this cap
    LibraAccount::pay_from<Coin1>(&cap, sender, 100, x"", x"");

    // restoring the capability should flip the flag back
    LibraAccount::restore_withdraw_capability(cap);
    assert(!LibraAccount::delegated_withdraw_capability(sender), 52);
}
}
