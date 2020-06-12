//! account: alice, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: association
script{
    use 0x1::LibraSystem;
    use 0x1::ValidatorConfig::{Self, DecertifyValidator};
    use 0x1::Roles;
    // Decertify two validators to make sure we can remove both
    // from the set and trigger reconfiguration
    fun main(account: &signer) {
        assert(LibraSystem::is_validator({{alice}}) == true, 98);
        assert(LibraSystem::is_validator({{vivian}}) == true, 99);
        assert(LibraSystem::is_validator({{viola}}) == true, 100);
        let cap = Roles::extract_privilege_to_capability<DecertifyValidator>(account);
        ValidatorConfig::decertify(&cap, {{vivian}});
        ValidatorConfig::decertify(&cap, {{alice}});
        Roles::restore_capability_to_privilege(account, cap);
        LibraSystem::update_and_reconfigure(account);
        assert(LibraSystem::is_validator({{alice}}) == false, 101);
        assert(LibraSystem::is_validator({{vivian}}) == false, 102);
        assert(LibraSystem::is_validator({{viola}}) == true, 103);
    }
}

// check: EXECUTED
// check: NewEpochEvent

//! new-transaction
//! sender: viola
// validators: viola
script{
    use 0x1::LibraSystem;
    use 0x1::ValidatorConfig;
    // Two reconfigurations cannot happen in the same block
    fun main(account: &signer) {
        ValidatorConfig::set_consensus_pubkey(account, {{viola}}, x"40");
        LibraSystem::update_and_reconfigure(account);

        ValidatorConfig::set_consensus_pubkey(account, {{viola}}, x"50");
        LibraSystem::update_and_reconfigure(account);
    }
}

// check: ABORT
// check: 23
