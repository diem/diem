script {
fun main() {
    let x = 0;
    let y;
    let b = true;
    while (copy b) {
        y = move x;
        y;
        b = false
    }
}
}
// TODO: fix verifier remove identical errors
// check: MOVELOC_UNAVAILABLE_ERROR
// check: MOVELOC_UNAVAILABLE_ERROR
