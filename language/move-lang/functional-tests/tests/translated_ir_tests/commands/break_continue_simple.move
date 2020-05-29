script {
fun main() {
    let x = 0;
    while (true) {
        if (x >= 5) break;
        x = x + 1;
        continue
    };
    0x0::Transaction::assert(move x == 5, 42);
}
}
