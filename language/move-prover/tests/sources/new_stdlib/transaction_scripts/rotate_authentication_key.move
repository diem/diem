use 0x0::LibraAccount;
fun main(new_key: vector<u8>) {
  LibraAccount::rotate_authentication_key(new_key)
}
