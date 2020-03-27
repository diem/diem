fun main() {
    if (true) {
        loop { break }
    } else {
        0x0::Transaction::assert(false, 42);
        return ()
    }
}
