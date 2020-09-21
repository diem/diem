//! max-gas: 1000

script {
    fun main(_s: &signer) {
        loop ()
    }
}

// check: "EXECUTION_FAILURE { status_code: OUT_OF_GAS"
