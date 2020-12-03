//! account: bob, 0, 0, vasp

//! sender: diemroot
//! execute-as: bob
script {
use 0x1::Signer;
use 0x1::CoreAddresses;
fun main(dr: &signer, bob: &signer) {
    assert(Signer::address_of(dr) == CoreAddresses::DIEM_ROOT_ADDRESS(), 0);
    assert(Signer::address_of(bob) == {{bob}}, 1);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
//! execute-as: bob
script {
use 0x1::Signer;
use 0x1::CoreAddresses;
fun main(dr: &signer, bob: &signer) {
    assert(Signer::address_of(dr) == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), 0);
    assert(Signer::address_of(bob) == {{bob}}, 1);
}
}
// check: REJECTED_WRITE_SET
