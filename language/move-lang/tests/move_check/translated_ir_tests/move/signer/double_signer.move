
script {
    fun t0(_s: signer, _s2: signer) {
    }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE

script {
    fun t1(_s: signer, _s2: signer, _u: u64) {
    }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE

script {
    fun t2(_s: signer, _u: u64, _s2: signer) {
    }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE
