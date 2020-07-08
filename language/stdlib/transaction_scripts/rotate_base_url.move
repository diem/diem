script {
use 0x1::DualAttestation;

/// Rotate `account`'s base URL to `new_url`.
fun rotate_base_url(account: &signer, new_url: vector<u8>) {
    DualAttestation::rotate_base_url(account, new_url)
}
}
