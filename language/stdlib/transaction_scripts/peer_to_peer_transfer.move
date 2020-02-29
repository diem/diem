fun main(payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  0x0::LibraAccount::pay_from_sender(payee, auth_key_prefix, amount)
}
