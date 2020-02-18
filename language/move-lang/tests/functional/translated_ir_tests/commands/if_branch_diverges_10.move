fun main() {
    if (true) {
        loop return ()
    } else {
        0x0::Transaction::assert(false, 42);
        return ()
    }
}
