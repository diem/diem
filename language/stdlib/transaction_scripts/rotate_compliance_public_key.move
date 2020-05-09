script {
use 0x0::LibraAccount;

fun main(vasp: &signer, new_key: vector<u8>) {
    LibraAccount::rotate_compliance_public_key(vasp, new_key)
}
}
