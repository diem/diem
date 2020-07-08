script {
use 0x1::DualAttestation;

/// Encode a program that rotates `account`'s compliance public key to `new_key`.
fun rotate_compliance_public_key(account: &signer, new_key: vector<u8>) {
    DualAttestation::rotate_compliance_public_key(account, new_key)
}
}
