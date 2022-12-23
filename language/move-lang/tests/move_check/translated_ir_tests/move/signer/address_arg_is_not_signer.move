address 0x2 {
module M {
    struct R has key { s: signer }
    public fun store_signer(s1: &signer, s: signer) {
        move_to(s1, R { s })
    }
}
}

script {
    fun t1(s: &signer) {
        0x2::M::store_signer(s)
    }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE

script {
    fun t2(_s: signer, s2: &signer) {
        0x2::M::store_signer(s2)
    }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE

script {
    fun t3(_s: signer, _s2: signer) { }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE
