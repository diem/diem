fun main (payee: address, amount: u64, metadata: vector<u8>) {
  0x0::LibraAccount::pay_from_sender_with_metadata(payee, amount, metadata)
}
