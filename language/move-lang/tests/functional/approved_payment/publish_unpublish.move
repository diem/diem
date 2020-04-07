// Publishing and unpublishing should work

//! account: alice

//! sender: alice
use 0x0::ApprovedPayment;
use 0x0::Transaction;
fun main() {
    Transaction::assert(!ApprovedPayment::exists({{alice}}), 6001);
    let pubkey = x"aa306695ca5ade60240c67b9b886fe240a6f009b03e43e45838334eddeae49fe";
    ApprovedPayment::publish(pubkey);
    Transaction::assert(ApprovedPayment::exists({{alice}}), 6002);
    ApprovedPayment::unpublish_from_sender();
    Transaction::assert(!ApprovedPayment::exists({{alice}}), 6003);
}

// check: EXECUTED
