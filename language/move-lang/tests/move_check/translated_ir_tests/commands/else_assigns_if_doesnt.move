script {
fun main() {
    let x;
    let y;
    if (true) {
        y = 0;
    } else {
        x = 42;
        x;
    };
    0x0::Transaction::assert(y == 0, 42);
}
}

// check: COPYLOC_UNAVAILABLE_ERROR
