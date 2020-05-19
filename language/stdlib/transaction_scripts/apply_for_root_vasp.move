script {
use 0x0::VASP;
fun main(
    human_name: vector<u8>,
    base_url: vector<u8>,
    travel_rule_public_key: vector<u8>
) {
    VASP::apply_for_vasp_root_credential(human_name, base_url, travel_rule_public_key);
}
}
