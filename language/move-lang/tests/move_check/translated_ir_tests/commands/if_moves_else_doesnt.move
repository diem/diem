script {
fun main() {
    let x = 0;
    let y = if (true) move x else 0;
    y;
    assert(x == 0, 42);
}
}

// check: COPYLOC_UNAVAILABLE_ERROR
