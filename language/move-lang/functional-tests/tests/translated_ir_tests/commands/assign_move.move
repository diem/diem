script {
fun main() {
    let x;
    let y;

    x = 5;
    y = move x;
    assert(y == 5, 42);
}
}
