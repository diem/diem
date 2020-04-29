script {
fun main() {
    let x = 5;
    let ref;
    if (true) {
        ref = &x;
    };
    0x0::Transaction::assert(*move ref == 5, 42);
}
}

// check: MOVELOC_UNAVAILABLE_ERROR
