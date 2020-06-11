//! account: bob, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: alice

//! new-transaction
script {
use 0x1::LibraSystem;
use 0x1::LibraConfig::CreateOnChainConfig;
use 0x1::Roles;
fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<CreateOnChainConfig>(account);
    LibraSystem::initialize_validator_set(account, &r);
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: ABORTED
// check: 1

//! new-transaction
script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::update_and_reconfigure(&assoc_root_role);
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}
// check: ABORTED
// check: 22

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_operator(account, 0x0);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::Signer;
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_operator(account, Signer::address_of(account))
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    // delegate to alice
    fun main(account: &signer) {
        ValidatorConfig::set_operator(account, {{alice}});
        ValidatorConfig::remove_operator(account);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{vivian}},
                                    x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
                                    x"", x"", x"", x"");
    }
}
// check: ABORTED
// check: 1101

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{vivian}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"", x"", x"");
    }
}
// check: ABORTED
// check: 1101
