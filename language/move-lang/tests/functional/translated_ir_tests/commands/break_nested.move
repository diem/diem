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
    0x0::Transaction::assert(x == 3, 42);
    0x0::Transaction::assert(y == 5, 42);
}
}
