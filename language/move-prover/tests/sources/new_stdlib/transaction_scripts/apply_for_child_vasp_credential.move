use 0x0::VASP;
fun main(root_vasp_address: address) {
    VASP::apply_for_child_vasp_credential(root_vasp_address)
}
