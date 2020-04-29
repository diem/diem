script {
fun main() {
    let x;
    let y;
    if (true) {
        x = 42;
    } else {
        y = 0;
        y;
    };
    0x0::Transaction::assert(x == 42, 42);
}
}

// check: COPYLOC_UNAVAILABLE_ERROR
