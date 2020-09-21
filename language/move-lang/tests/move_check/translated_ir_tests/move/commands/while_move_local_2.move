script {
fun main() {
    let x = 0;
    let y = 0;
    let b = true;
    while (true) {
        if (b) {
            y = move x;
        } else {
            x = move y;
        };
        b = false;
    }
}
}
// TODO: fix verifier remove identical errors
// check: MOVELOC_UNAVAILABLE_ERROR
// check: MOVELOC_UNAVAILABLE_ERROR
