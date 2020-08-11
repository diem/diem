//! account: bob, 0, 0, vasp

//! sender: libraroot
//! execute-as: bob
script {
use 0x1::Signer;
use 0x1::CoreAddresses;
fun main(lr: &signer, bob: &signer) {
    assert(Signer::address_of(lr) == CoreAddresses::LIBRA_ROOT_ADDRESS(), 0);
    assert(Signer::address_of(bob) == {{bob}}, 1);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
//! execute-as: bob
script {
use 0x1::Signer;
use 0x1::CoreAddresses;
fun main(lr: &signer, bob: &signer) {
    assert(Signer::address_of(lr) == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), 0);
    assert(Signer::address_of(bob) == {{bob}}, 1);
}
}
// check: REJECTED_WRITE_SET
