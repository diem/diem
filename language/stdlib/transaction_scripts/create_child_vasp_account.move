script {
use 0x0::LibraAccount;
fun main<CoinType>(
    parent_vasp: &signer,
    child_address: address,
    auth_key_prefix: vector<u8>,
    add_all_currencies: bool
) {
    LibraAccount::create_child_vasp_account<CoinType>(
        parent_vasp,
        child_address,
        auth_key_prefix,
        add_all_currencies,
    )
}
}
