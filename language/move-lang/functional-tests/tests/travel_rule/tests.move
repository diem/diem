//! account: freddy_mac

//! new-transaction
//! sender: freddy_mac
script{
    use 0x1::DualAttestationLimit;
    fun main() {
        DualAttestationLimit::get_cur_microlibra_limit();
    }
}
// check: EXECUTED

//! new-transaction
script{
    use 0x1::DualAttestationLimit::{Self};
    fun main(not_blessed: &signer) {
        // Roles::restore_capability_to_privilege(not_blessed, r)
        DualAttestationLimit::set_microlibra_limit(not_blessed, 99);
    }
}
// check: ABORTED
// check: 3

// too low limit

//! new-transaction
//! sender: blessed
script{
    use 0x1::DualAttestationLimit::{Self};
    fun main(not_blessed: &signer) {
        // Roles::restore_capability_to_privilege(not_blessed, r)
        DualAttestationLimit::set_microlibra_limit(not_blessed, 999);
    }
}
// check: ABORTED

//! new-transaction
//! sender: blessed
script{
    use 0x1::DualAttestationLimit::{Self};
    fun main(not_blessed: &signer) {
        // Roles::restore_capability_to_privilege(not_blessed, r)
        DualAttestationLimit::set_microlibra_limit(not_blessed, 1001);
    }
}
// check: EXECUTED
