module Holder {
    use 0x0::RegisteredCurrencies;
    resource struct Holder {
        cap: RegisteredCurrencies::RegistrationCapability,
    }
    public fun hold(account: &signer, cap: RegisteredCurrencies::RegistrationCapability) {
        move_to<Holder>(account, Holder { cap })
    }
}
//! new-transaction
script {
    use {{default}}::Holder;
    use 0x0::RegisteredCurrencies;
    fun main(account: &signer) {
        Holder::hold(account, RegisteredCurrencies::initialize(account));
    }
}
// check: ABORTED
// check: 0
