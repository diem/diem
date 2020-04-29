script {
use 0x0::VASP;
fun main(child_address: address) {
    VASP::grant_parent_capability(child_address)
}
}
