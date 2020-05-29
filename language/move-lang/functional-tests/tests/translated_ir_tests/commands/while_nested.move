script {
fun main() {
    let x = 0;
    let z = 0;
    let y;
    while (x < 3) {
        x = x + 1;
        y = 0;
        while (y < 7) {
            y = y + 1;
            z = z + 1;
        }
    };
    0x0::Transaction::assert(z == 21, 42)
}
}
