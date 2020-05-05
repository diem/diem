script {
use 0x0::LibraAccount;

fun main<Token>(fresh_address: address, auth_key_prefix: vector<u8>) {
  LibraAccount::create_account<Token>(fresh_address, auth_key_prefix);
}
}
