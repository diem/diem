fun main() {
    let x;
    if (true) {
        return ()
    } else {
        x = 0
    };
    0x0::Transaction::assert(x == 0, 42);
}
