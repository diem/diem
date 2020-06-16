script {
use 0x1::VASP;

fun rotate_base_url(vasp: &signer, new_url: vector<u8>) {
    VASP::rotate_base_url(vasp, new_url)
}
}
