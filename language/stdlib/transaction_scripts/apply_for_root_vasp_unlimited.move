use 0x0::VASP;
use 0x0::AccountLimits;
fun main(
    human_name: vector<u8>,
    base_url: vector<u8>,
    ca_cert: vector<u8>,
) {
    VASP::apply_for_vasp_root_credential(human_name, base_url, ca_cert);
    AccountLimits::publish_unrestricted_limits();
}
