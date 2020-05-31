script {
use 0x0::LibraAccount;

fun main<Token>(
    payee: address,
    amount: u64,
    metadata: vector<u8>,
    metadata_signature: vector<u8>
) {
  LibraAccount::pay_from_sender_with_metadata<Token>(payee, amount, metadata, metadata_signature)
}
}
