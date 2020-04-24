use 0x0::VASP;
fun main(parent_address: address) {
    VASP::remove_parent_capability(parent_address)
}
