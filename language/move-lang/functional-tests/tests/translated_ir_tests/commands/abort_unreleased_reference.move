script {
fun main() {
    let x = 0;
    let x_ref = &mut x;
    copy x_ref;
    // x_ref does not have to be released
    abort 0
}
}

// not: VerificationFailure

// check: ABORTED
// check: 0
