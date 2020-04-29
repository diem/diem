script {
fun main() {
    let x;
    let y;
    x = 5;
    y = x;
    0x0::Transaction::assert(y == 5, 42);
}
}
