script {
use 0x1::LibraAccount;
fun create_parent_vasp_account<CoinType>(
    association: &signer,
    new_account_address: address,
    auth_key_prefix: vector<u8>,
    human_name: vector<u8>,
    base_url: vector<u8>,
    compliance_public_key: vector<u8>,
    add_all_currencies: bool
) {
    LibraAccount::create_parent_vasp_account<CoinType>(
        association,
        new_account_address,
        auth_key_prefix,
        human_name,
        base_url,
        compliance_public_key,
        add_all_currencies
    )
}
}
