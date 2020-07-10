//! account: freddy_mac

//! new-transaction
//! sender: freddy_mac
script{
    use 0x1::DualAttestation;
    fun main() {
        DualAttestation::get_cur_microlibra_limit();
    }
}
// check: EXECUTED

//! new-transaction
script{
    use 0x1::DualAttestation::{Self};
    fun main(not_blessed: &signer) {
        DualAttestation::set_microlibra_limit(not_blessed, 99);
    }
}
// check: ABORTED
// check: 3

//! new-transaction
//! sender: blessed
script{
    use 0x1::DualAttestation::{Self};
    fun main(not_blessed: &signer) {
        DualAttestation::set_microlibra_limit(not_blessed, 1001);
    }
}
// check: EXECUTED
