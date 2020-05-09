script {
use 0x0::LibraAccount;
fun main<CoinType>(parent_vasp: &signer, child_address: address, auth_key_prefix: vector<u8>) {
    LibraAccount::create_child_vasp_account<CoinType>(child_address, auth_key_prefix, parent_vasp)
}
}
