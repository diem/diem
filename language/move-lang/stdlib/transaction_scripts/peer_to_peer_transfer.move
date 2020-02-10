fun main(payee: address, amount: u64) {
  0x0::LibraAccount::pay_from_sender(payee, amount);
}
