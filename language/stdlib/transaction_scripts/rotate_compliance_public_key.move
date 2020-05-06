script {
use 0x0::VASP;

fun main(root_vasp_address: address, new_compliance_public_key: vector<u8>) {
    VASP::rotate_compliance_public_key(root_vasp_address, new_compliance_public_key)
}
}
