script {
use 0x1::VASP;

/// Encode a program that rotates `vasp_root_addr`'s compliance public key to `new_key`.
fun rotate_compliance_public_key(vasp: &signer, new_key: vector<u8>) {
    VASP::rotate_compliance_public_key(vasp, new_key)
}
}
