//! account: alice, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator

//! new-transaction
//! sender: alice
script{
    use 0x1::ValidatorConfig;
    // rotate alice's pubkey
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{alice}},
                                    x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
                                    x"", x"", x"", x"");
    }
}

// check: events: []
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

// not: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: vivian
script{
    use 0x1::ValidatorConfig;

    // rotate vivian's pubkey and then run the block prologue. Now, reconfiguration should be triggered.
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{vivian}},
                                    x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
                                    x"", x"", x"", x"");
    }
}

// check: EXECUTED

//! new-transaction
//! sender: association
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    use 0x1::ValidatorConfig;

    // rotate vivian's pubkey and then run the block prologue. Now, reconfiguration should be triggered.
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::update_and_reconfigure(&assoc_root_role);
        Roles::restore_capability_to_privilege(account, assoc_root_role);
        // check that the validator set contains Vivian's new key after reconfiguration
        assert(*ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{vivian}})) ==
               x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", 98);
    }
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: vivian
script{
    use 0x1::ValidatorConfig;
    // rotate vivian's pubkey to the same value.
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{vivian}},
                                    x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
                                    x"", x"", x"", x"");
    }
}

// not: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: association
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    // No reconfiguration should be
    // triggered. the not "NewEpochEvent" check part tests this because reconfiguration always emits a
    // NewEpoch event.
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::update_and_reconfigure(&assoc_root_role);
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// not: NewEpochEvent
// check: EXECUTED
