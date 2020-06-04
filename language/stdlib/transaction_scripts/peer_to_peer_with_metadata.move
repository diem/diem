script {
use 0x0::LibraAccount;

fun main<Token>(
    payer: &signer,
    payee: address,
    amount: u64,
    metadata: vector<u8>,
    metadata_signature: vector<u8>
) {
  LibraAccount::pay_from_with_metadata<Token>(payer, payee, amount, metadata, metadata_signature)
}
}
