fun main(payee: address, amount: u64) {
  0x0::LibraAccount::mint_to_address(payee, amount);
}
