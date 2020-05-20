script {
use 0x0::LBR;
use 0x0::LibraAccount;
fun main(payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  if (!LibraAccount::exists(payee)) {
      LibraAccount::create_testnet_account<LBR::T>(payee, auth_key_prefix);
  };
  LibraAccount::mint_lbr_to_address(payee, amount);
}
}
