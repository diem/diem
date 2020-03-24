// Test that publishing a key with an invalid length or rotating to a key with an invalid length
// causes failures.

//! account: alice

//! sender: alice
use 0x0::ApprovedPayment;
fun main() {
    let invalid_pubkey = x"aa"; // too short
    ApprovedPayment::publish(invalid_pubkey)
}

// check: ABORTED
// check: 9003

// publish with a valid pubkey...

//! new-transaction
//! sender: alice
use 0x0::ApprovedPayment;
fun main() {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    ApprovedPayment::publish(pubkey)
}

// check: EXECUTED

// ... but then rotate to an invalid one

//! new-transaction
//! sender: alice
use 0x0::ApprovedPayment;
fun main() {
    let invalid_pubkey = x"aa"; // too short
    ApprovedPayment::rotate_sender_key(invalid_pubkey)
}

// check: ABORTED
// check: 9003
