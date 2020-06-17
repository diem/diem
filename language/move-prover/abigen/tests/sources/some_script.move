script {
/// This script does really nothing but just aborts.
fun some<Token>(_account: &signer, code: u64) {
    abort code
}
}
