use 0x0::VASP;
use 0x0::AccountLimits;
fun main(root_vasp_addr: address) {
    VASP::grant_vasp(root_vasp_addr);
    AccountLimits::certify_limits_definition(root_vasp_addr);
}
