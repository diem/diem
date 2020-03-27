fun main() {
    let x = 0;
    if (true) {
        let y = move x;
        y;
    };
    0x0::Transaction::assert(x == 0, 42);
}

// check: COPYLOC_UNAVAILABLE_ERROR
