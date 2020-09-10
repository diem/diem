script {
fun main(s: &signer) {
    assert(s == s, 42)
}
}
