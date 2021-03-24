script {
fun main(s: signer) {
    let s = &s;
    assert(s == s, 42)
}
}
