//! account: bob, 10000LBR, 0, empty
//! account: alice, 10000LBR, 0, unhosted

module Holder {
    resource struct Hold<T> { x: T }
    public fun hold<T>(x: T) {
        move_to_sender(Hold<T>{x})
    }
}

//! new-transaction
script {
    use 0x0::AccountTrack;
    fun main() {
        AccountTrack::initialize();
    }
}
// check: ABORTED
// check: 3000

//! new-transaction
script {
    use 0x0::AccountTrack;
    use {{default}}::Holder;
    fun main() {
        Holder::hold(
            AccountTrack::grant_calling_capability()
        )
    }
}
// check: ABORTED
// check: 3000

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{alice}}, 1);
    }
}
// check: ABORTED
// check: 3002

//! new-transaction
//! sender: alice
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{bob}}, 1);
    }
}
// check: ABORTED
// check: 3002
