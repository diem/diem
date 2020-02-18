fun main() {
    if (true) {
        return ()
    } else {
        0x0::Transaction::assert(false, 42);
    };
    0x0::Transaction::assert(false, 43);
}
