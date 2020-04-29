script {
use 0x0::ApprovedPayment;

// Deposit `amount` `Token`s in `payee`'s account if the `signature` on the payment metadata matches
// the public key stored in the `payee`'s ApprovedPayment` resource.
// Aborts if the signature does not match, `payee` does not have an `ApprovedPayment` resource,
// or the sender's balance is less than `amount`.
fun main<Token>(
    payee: address,
    amount: u64,
    metadata: vector<u8>,
    signature: vector<u8>
) {
    ApprovedPayment::deposit_to_payee<Token>(payee, amount, metadata, signature)
}
}
