fun main() {
    let x: u64;
    let y: u64;

    let x;
    let y;
    if (true) {
        x = 1;
        y = move x;
        y;
    } else {
        x = 0;
    };
    0x0::Transaction::assert(x == 5, 42);
}

// check: COPYLOC_UNAVAILABLE_ERROR
