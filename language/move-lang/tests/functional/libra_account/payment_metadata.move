//! account: alice
//! account: bob

//! sender: alice
use 0x0::LBR;
use 0x0::LibraAccount;
// send a transaction with metadata and make sure we see it in the PaymentReceivedEvent
fun main() {
    LibraAccount::pay_from_sender_with_metadata<LBR::T>({{bob}}, 1000, x"deadbeef", x"");
}

// check: SentPaymentEvent
// check: deadbeef
// check: ReceivedPaymentEvent
// check: deadbeef
// check: EXECUTED

//! new-transaction
//! sender: alice
use 0x0::LibraAccount;
use 0x0::LBR;
// same thing, but using "deposit_with_metadata" API
fun main() {
    LibraAccount::deposit_with_metadata<LBR::T>(
        {{bob}},
        LibraAccount::withdraw_from_sender<LBR::T>(100),
        x"deadbeef",
        x""
    )
}

// check: SentPaymentEvent
// check: deadbeef
// check: ReceivedPaymentEvent
// check: deadbeef
// check: EXECUTED
