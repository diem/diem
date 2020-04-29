script {
fun main() {
    let x;
    if (true) x = 42;
    0x0::Transaction::assert(x == 42, 42);
}
}

// check: COPYLOC_UNAVAILABLE_ERROR
