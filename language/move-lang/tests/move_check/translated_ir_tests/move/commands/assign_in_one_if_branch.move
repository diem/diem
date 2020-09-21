script {
fun main() {
    let x;
    let y;
    if (true) x = 5 else ();
    if (true) y = 5;
    x == y;
}
}

// check: COPYLOC_UNAVAILABLE_ERROR
