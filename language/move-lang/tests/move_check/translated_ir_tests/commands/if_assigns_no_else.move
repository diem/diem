script {
fun main() {
    let x;
    if (true) x = 42;
    assert(x == 42, 42);
}
}

// check: COPYLOC_UNAVAILABLE_ERROR
