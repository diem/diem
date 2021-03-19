script {
    fun t0(_u: u64, _s: signer) {
    }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE

script {
    fun t1(_u: u64, _s: signer, _u2: u64) {
    }
}
// check: INVALID_MAIN_FUNCTION_SIGNATURE
