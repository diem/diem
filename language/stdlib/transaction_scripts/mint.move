script {
use 0x1::LBR;
use 0x1::LibraAccount;
fun main<Token>(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  if (!LibraAccount::exists_at(payee)) {
      LibraAccount::create_testnet_account<Token>(account, payee, auth_key_prefix)
  };
  if (LBR::is_lbr<Token>()) {
      LibraAccount::mint_lbr_to_address(account, payee, amount);
  } else {
      LibraAccount::mint_to_address<Token>(account, payee, amount)
  }
}
}
