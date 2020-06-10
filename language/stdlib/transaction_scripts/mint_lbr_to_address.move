script {
use 0x0::LBR::LBR;
use 0x0::LibraAccount;
fun main(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  if (!LibraAccount::exists(payee)) {
      LibraAccount::create_testnet_account<LBR>(account, payee, auth_key_prefix);
  };
  LibraAccount::mint_lbr_to_address(account, payee, amount);
}
}
