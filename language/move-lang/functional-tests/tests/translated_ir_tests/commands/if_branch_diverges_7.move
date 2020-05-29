script {
fun main() {
    if (true) {
        loop { if (true) return () else continue }
    } else {
        0x0::Transaction::assert(false, 42);
        return ()
    }
}
}
