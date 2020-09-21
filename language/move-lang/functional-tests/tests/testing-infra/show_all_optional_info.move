//! show-gas: true
//! show-writeset: true
//! show-events: true
script {
    fun main() {}
}


//! new-transaction
//! show-gas: true
//! show-writeset: true
//! show-events: true
script {
    fun main() {
        _ = 10 - 20;
    }
}
