script {
fun main() {
    if (true) {
        loop { if (true) return () else continue }
    } else {
        assert(false, 42);
        return ()
    }
}
}
