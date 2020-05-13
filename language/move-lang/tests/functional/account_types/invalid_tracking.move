//! new-transaction
module Holder {
    resource struct H<T> {
        x: T,
    }

    public fun hold<T>(x: T) {
        move_to_sender(H<T>{x})
    }
}

//! new-transaction
script {
use {{default}}::Holder;
use 0x0::AccountType;
fun main() {
    Holder::hold(
        AccountType::grant_account_tracking()
    );
}
}
// check: ABORTED
// check: 2006
