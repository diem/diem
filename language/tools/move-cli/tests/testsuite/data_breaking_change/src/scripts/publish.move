script {
use 0x2::M;
fun publish(account: signer) {
    M::publish(&account)
}
}
