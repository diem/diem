//! account: alice
//! account: bob

//! sender: alice
script {
use 0x1::LBR::LBR;
use 0x1::LibraAccount;
// send a transaction with metadata and make sure we see it in the PaymentReceivedEvent
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from_with_metadata<LBR>(&with_cap, {{bob}}, 1000, x"deadbeef", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}

// check: SentPaymentEvent
// check: deadbeef
// check: ReceivedPaymentEvent
// check: deadbeef
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
use 0x1::LBR::LBR;
// same thing, but using "deposit_with_metadata" API
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::deposit_with_metadata<LBR>(
        account,
        {{bob}},
        LibraAccount::withdraw_from<LBR>(&with_cap, 100),
        x"deadbeef",
        x""
    );
    LibraAccount::restore_withdraw_capability(with_cap);
}
}

// check: SentPaymentEvent
// check: deadbeef
// check: ReceivedPaymentEvent
// check: deadbeef
// check: EXECUTED
