script {
fun main() {
    let x = 0;
    let y = 0;
    loop {
        if (x < 10) {
            x = x + 1;
            if (x % 2 == 0) continue;
            y = y + x
        } else {
            break
        }
    };
    assert(y == 25, 42);
}
}
