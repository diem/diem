module Holder {
    use 0x1::RegisteredCurrencies;
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
    use 0x1::RegisteredCurrencies;
    use 0x1::LibraConfig::CreateOnChainConfig;
    use 0x1::Roles;
    fun main(account: &signer) {
        let r = Roles::extract_privilege_to_capability<CreateOnChainConfig>(account);
        Holder::hold(account, RegisteredCurrencies::initialize(account, &r));
        Roles::restore_capability_to_privilege(account, r);
    }
}
// check: ABORTED
// check: 0
