script {
fun main() {
    let x = 0;
    let y = 0;
    while (true) {
        loop {
            y = 5;
            break
        };
        x = 3;
        break
    };
    assert(x == 3, 42);
    assert(y == 5, 42);
}
}
