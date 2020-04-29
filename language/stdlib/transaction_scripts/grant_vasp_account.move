script {
use 0x0::VASP;
fun main(root_vasp_addr: address) {
    VASP::grant_vasp(root_vasp_addr);
}
}
