use 0x0::LBR;

fun main (payee: address, auth_key_prefix: vector<u8>, amount: u64, metadata: vector<u8>) {
  0x0::LibraAccount::pay_from_sender_with_metadata<LBR::T>(payee, auth_key_prefix, amount, metadata)
}
