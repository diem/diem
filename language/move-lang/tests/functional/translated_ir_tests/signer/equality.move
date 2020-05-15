script {
fun main(s: &signer) {
    0x0::Transaction::assert(s == s, 42)
}
}
