script {
fun main() {
    if (true) return ();
    0x0::Transaction::assert(false, 42);
}
}
