script {
fun main() {
    let x;
    let y;
    if (true) {
        x = 1;
        y = move x;
        x = 5;
        y;
    } else {
        x = 0;
    };
    assert(copy x == 5, 42);
}
}
