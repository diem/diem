//! max-gas: 700
script {
    fun main(_s: signer) {
        loop ()
    }
}
// check: "EXECUTION_FAILURE { status_code: OUT_OF_GAS, location: Script,"
// check: "Keep(OUT_OF_GAS)"
