script {
fun main() {
    if (true) {
        loop return ()
    } else {
        assert(false, 42);
        return ()
    }
}
}
