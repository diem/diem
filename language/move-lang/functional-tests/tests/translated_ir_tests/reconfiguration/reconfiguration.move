//! account: alice, 1000000, 0
//! account: vivian, 1000000, 0, validator
//! account: valentina, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
// Reconfiguration can only be invoked by association.
script {
use 0x1::LibraConfig;
use 0x1::Roles::{Self, AssociationRootRole};

fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
    LibraConfig::reconfigure(&r);
    Roles::restore_capability_to_privilege(account, r);
}
}

// check: ABORT
// check: 1

//! new-transaction
//! sender: association
script {
use 0x1::LibraConfig;
use 0x1::Roles::{Self, AssociationRootRole};

fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
    LibraConfig::reconfigure(&r);
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: association
// Cannot trigger two reconfiguration within the same block.
script {
use 0x1::LibraConfig;
use 0x1::Roles::{Self, AssociationRootRole};

fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
    LibraConfig::reconfigure(&r);
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: ABORTED
// check: 23

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
script {
use 0x1::LibraConfig;
use 0x1::Roles::{Self, AssociationRootRole};

fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
    LibraConfig::reconfigure(&r);
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: NewEpochEvent
// check: EXECUTED
