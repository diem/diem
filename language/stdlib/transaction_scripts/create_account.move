use 0x0::LibraAccount;

// TODO: remove initial_amount?
fun main<Token>(fresh_address: address, auth_key_prefix: vector<u8>, initial_amount: u64) {
  LibraAccount::create_unhosted_account<Token>(fresh_address, auth_key_prefix);
  if (initial_amount > 0) LibraAccount::deposit(
        fresh_address,
        LibraAccount::withdraw_from_sender<Token>(initial_amount)
     );
}
