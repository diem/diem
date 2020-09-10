script {
fun main() {
    let x = 0;
    if (true) {
        let y = move x;
        y;
    };
    assert(x == 0, 42);
}
}

// check: COPYLOC_UNAVAILABLE_ERROR
