address 0x1 {
module M {
    resource struct R { s: signer }
    public fun store_signer(s: signer) {
        move_to_sender(R { s })
    }
}
}

script {
    fun t1(s: signer) {
        0x1::M::store_signer(s)
    }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE

script {
    fun t2(s: &signer, s2: signer) {
        0x1::M::store_signer(s2)
    }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE

script {
    fun t3(s: &signer, s2: &signer) { }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE
