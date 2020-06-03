script {
use 0x0::LibraAccount;
fun main(account: &signer, new_key: vector<u8>) {
  LibraAccount::rotate_authentication_key(account, new_key)
}
}
