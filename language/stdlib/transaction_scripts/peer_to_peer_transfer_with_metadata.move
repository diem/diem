fun main (payee: address, amount: u64, metadata: bytearray) {
  0x0::LibraAccount::pay_from_sender_with_metadata(payee, amount, metadata)
}
