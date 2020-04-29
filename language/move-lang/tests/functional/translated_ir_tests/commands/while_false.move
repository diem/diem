script {
fun main() {
    let x = 0;
    while (false) x = 1;
    0x0::Transaction::assert(x == 0, 42);
}
}
