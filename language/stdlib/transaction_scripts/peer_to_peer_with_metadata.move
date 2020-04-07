use 0x0::LibraAccount;

fun main<Token>(payee: address, auth_key_prefix: vector<u8>, amount: u64, metadata: vector<u8>) {
  LibraAccount::pay_from_sender_with_metadata<Token>(payee, auth_key_prefix, amount, metadata)
}
