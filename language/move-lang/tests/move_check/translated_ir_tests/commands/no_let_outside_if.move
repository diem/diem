fun main() {
    if (true) {
        y = 5;
    } else {
        y = 0;
    };
    0x0::Transaction::assert(y == 5, 42);
}
