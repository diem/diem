//! account: dawn, 0LBR

module Holder {
    resource struct H<T> { x: T }
    public fun hold<T>(x: T) {
        move_to_sender(H<T>{x})
    }
}

//! new-transaction
//! sender: association
//! max-gas: 1000000
script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main() {
    LibraAccount::deposit<LBR::T>({{dawn}}, LBR::mint(200));
}
}
// check: EXECUTED

// publish preburn resource for LBR to alice's account
//! new-transaction
//! sender: dawn
//! max-gas: 1000000
script {
use 0x0::LBR;
use 0x0::Libra;
use 0x0::LibraAccount;
fun main() {
    Libra::publish_preburn(Libra::new_preburn<LBR::T>());
    let coin = LibraAccount::withdraw_from_sender<LBR::T>(100);
    Libra::preburn_to_sender<LBR::T>(coin);
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
//! max-gas: 1000000
script {
use 0x0::LBR;
use 0x0::Libra;
use 0x0::LibraAccount;
use {{default}}::Holder;
fun main() {
    let burn_cap = Libra::grant_burn_capability<LBR::T>();
    LibraAccount::deposit<LBR::T>(
        {{dawn}},
        Libra::cancel_burn_with_capability<LBR::T>({{dawn}}, &burn_cap)
    );
    Holder::hold(burn_cap);
}
}
// check: EXECUTED
