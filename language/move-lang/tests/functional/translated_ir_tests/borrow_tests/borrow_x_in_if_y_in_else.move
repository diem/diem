fun main() {
    let x = 1;
    let y = 2;

    let ref;
    if (true) {
        ref = &x;
    } else {
        ref = &y;
    };
    0x0::Transaction::assert(*ref == 1, 42);
}
