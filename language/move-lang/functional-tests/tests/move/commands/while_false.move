script {
fun main() {
    let x = 0;
    while (false) x = 1;
    assert(x == 0, 42);
}
}
