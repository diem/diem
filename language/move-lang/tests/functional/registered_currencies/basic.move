module Holder {
    use 0x0::RegisteredCurrencies;
    resource struct Holder {
        cap: RegisteredCurrencies::RegistrationCapability,
    }
    public fun hold(cap: RegisteredCurrencies::RegistrationCapability) {
        move_to_sender<Holder>(Holder { cap })
    }
}
//! new-transaction
script {
    use {{default}}::Holder;
    use 0x0::RegisteredCurrencies;
    fun main() {
        Holder::hold(RegisteredCurrencies::initialize());
    }
}
// check: ABORTED
// check: 0
