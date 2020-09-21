script {
fun main() {
    if (true) {
        loop { if (true) return () else break }
    } else {
        assert(false, 42);
        return ()
    }
}
}
