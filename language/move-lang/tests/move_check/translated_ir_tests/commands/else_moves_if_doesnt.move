script {
fun main() {
    let x = 0;
    let y = if (true) 0 else move x; y;
    assert(x == 0, 42);
}
}

// check: COPYLOC_UNAVAILABLE_ERROR
