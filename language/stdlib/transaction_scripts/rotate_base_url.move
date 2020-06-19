script {
use 0x1::VASP;

/// Rotate `vasp_root_addr`'s base URL to `new_url`.
fun rotate_base_url(vasp: &signer, new_url: vector<u8>) {
    VASP::rotate_base_url(vasp, new_url)
}
}
