use 0x0::VASP;
use 0x0::AccountLimits;
fun main(
    // For VASP credential
    human_name: vector<u8>,
    base_url: vector<u8>,
    ca_cert: vector<u8>,
    // For account limits
    max_outflow: u64,
    max_inflow: u64,
    max_holding: u64,
    time_period: u64,
) {
    VASP::apply_for_vasp_root_credential(human_name, base_url, ca_cert);
    AccountLimits::publish_limits_definition(max_outflow, max_inflow, max_holding, time_period);
}
