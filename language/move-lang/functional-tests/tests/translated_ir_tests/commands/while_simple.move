script {
fun main() {
    let x = 0;
    while (x < 5) x = x + 1;
    0x0::Transaction::assert(x == 5, 42);
}
}
