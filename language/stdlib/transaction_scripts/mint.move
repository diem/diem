script {
use 0x0::LibraAccount;
fun main<Token>(payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  if (!LibraAccount::exists(payee)) LibraAccount::create_testnet_account<Token>(payee, auth_key_prefix);
  LibraAccount::mint_to_address<Token>(payee, amount);
}
}
