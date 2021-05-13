script {
use DiemFramework::XUS::XUS;
use DiemFramework::DiemAccount;
use Std::Signer;
fun main(account: signer) {
    let account = &account;
    let sender = Signer::address_of(account);

    // by default, an account has not delegated its withdrawal capability
    assert(!DiemAccount::delegated_withdraw_capability(sender), 50);

    // make sure we report that the capability has been extracted
    let cap = DiemAccount::extract_withdraw_capability(account);
    assert(DiemAccount::delegated_withdraw_capability(sender), 51);

    // and the sender should be able to withdraw with this cap
    DiemAccount::pay_from<XUS>(&cap, sender, 100, x"", x"");

    // restoring the capability should flip the flag back
    DiemAccount::restore_withdraw_capability(cap);
    assert(!DiemAccount::delegated_withdraw_capability(sender), 52);
}
}
