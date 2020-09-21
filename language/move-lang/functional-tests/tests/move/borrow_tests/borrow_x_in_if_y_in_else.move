script {
fun main() {
    let x = 1;
    let y = 2;

    let ref;
    if (true) {
        ref = &x;
    } else {
        ref = &y;
    };
    assert(*ref == 1, 42);
}
}
