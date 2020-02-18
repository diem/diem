fun main() {
    let x = 0;
    while (true) {
        x = x + 1;
        break
    };
    0x0::Transaction::assert(x == 1, 42);
}
