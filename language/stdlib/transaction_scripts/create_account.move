use 0x0::LBR;

// TODO: remove initial_amount?
fun main(fresh_address: address, auth_key_prefix: vector<u8>, initial_amount: u64) {
  0x0::LibraAccount::create_new_account<LBR::T>(fresh_address, auth_key_prefix, initial_amount)
}
