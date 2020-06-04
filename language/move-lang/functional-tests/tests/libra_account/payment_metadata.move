//! account: alice
//! account: bob

//! sender: alice
script {
use 0x0::LBR;
use 0x0::LibraAccount;
// send a transaction with metadata and make sure we see it in the PaymentReceivedEvent
fun main(account: &signer) {
    LibraAccount::pay_from_with_metadata<LBR::T>(account, {{bob}}, 1000, x"deadbeef", x"");
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
use 0x0::LibraAccount;
use 0x0::LBR;
// same thing, but using "deposit_with_metadata" API
fun main(account: &signer) {
    LibraAccount::deposit_with_metadata<LBR::T>(
        account,
        {{bob}},
        LibraAccount::withdraw_from<LBR::T>(account, 100),
        x"deadbeef",
        x""
    )
}
}

// check: SentPaymentEvent
// check: deadbeef
// check: ReceivedPaymentEvent
// check: deadbeef
// check: EXECUTED
