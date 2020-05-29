script {
use 0x0::LibraAccount;

fun main(vasp: &signer, new_url: vector<u8>) {
    LibraAccount::rotate_base_url(vasp, new_url)
}
}
