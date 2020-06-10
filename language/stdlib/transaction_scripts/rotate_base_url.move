script {
use 0x0::VASP;

fun main(vasp: &signer, new_url: vector<u8>) {
    VASP::rotate_base_url(vasp, new_url)
}
}
