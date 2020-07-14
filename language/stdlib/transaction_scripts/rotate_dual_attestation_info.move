script {
use 0x1::DualAttestation;

/// Rotate `account`'s base URL to `new_url` and its compliance public key to `new_key`.
/// Aborts if `account` is not a ParentVASP or DesignatedDealer
/// Aborts if `new_key` is not a well-formed public key
fun rotate_dual_attestation_info(account: &signer, new_url: vector<u8>, new_key: vector<u8>) {
    DualAttestation::rotate_base_url(account, new_url);
    DualAttestation::rotate_compliance_public_key(account, new_key)
}
}
