use 0x0::VASP;
fun main(child_address: address) {
    VASP::decertify_child_account(child_address)
}
