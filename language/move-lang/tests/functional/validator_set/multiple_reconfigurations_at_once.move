//! account: alice, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: association
script{
    use 0x0::LibraAccount;
    use 0x0::LibraSystem;
    // Decertify two validators to make sure we can remove both
    // from the set and trigger reconfiguration
    fun main() {
        0x0::Transaction::assert(LibraSystem::is_validator({{alice}}) == true, 98);
        0x0::Transaction::assert(LibraSystem::is_validator({{vivian}}) == true, 99);
        0x0::Transaction::assert(LibraSystem::is_validator({{viola}}) == true, 100);
        LibraAccount::decertify<LibraAccount::ValidatorRole>({{vivian}});
        LibraAccount::decertify<LibraAccount::ValidatorRole>({{alice}});
        LibraSystem::update_and_reconfigure();
        0x0::Transaction::assert(LibraSystem::is_validator({{alice}}) == false, 101);
        0x0::Transaction::assert(LibraSystem::is_validator({{vivian}}) == false, 102);
        0x0::Transaction::assert(LibraSystem::is_validator({{viola}}) == true, 103);
    }
}

// check: EXECUTED
// check: NewEpochEvent

//! new-transaction
//! sender: viola
// validators: viola
script{
    use 0x0::LibraSystem;
    use 0x0::ValidatorConfig;
    // Two reconfigurations cannot happen in the same block
    fun main() {
        ValidatorConfig::set_consensus_pubkey({{viola}}, x"40");
        LibraSystem::update_and_reconfigure();

        ValidatorConfig::set_consensus_pubkey({{viola}}, x"50");
        LibraSystem::update_and_reconfigure();
    }
}

// check: ABORT
// check: 23
