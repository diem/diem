script {
fun main() {
    let x = 0;
    loop {
        x = x + 1;
        break
    };
    assert(x == 1, 42);
}
}
