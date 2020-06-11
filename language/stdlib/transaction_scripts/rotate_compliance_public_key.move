script {
use 0x1::VASP;

fun main(vasp: &signer, new_key: vector<u8>) {
    VASP::rotate_compliance_public_key(vasp, new_key)
}
}
