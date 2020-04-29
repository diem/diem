script {
use 0x0::ApprovedPayment;

// Publish a newly created `ApprovedPayment` resource under the sender's account with approval key
// `public_key`.
// Aborts if the sender already has a published `ApprovedPayment` resource.
fun main(public_key: vector<u8>) {
    ApprovedPayment::publish(public_key)
}
}
