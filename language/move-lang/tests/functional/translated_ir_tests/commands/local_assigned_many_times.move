script {
fun main() {
    let x = 5;
    let y = 2;
    let x_ref;
    let y_ref;

    x_ref = &x;
    y_ref = &y;
    y_ref;
    x_ref;

    x_ref = &x;
    y_ref = x_ref;

    if (true) {
        _ = y_ref;
        x_ref = &y;
        y_ref = &x;
    } else {
        _ = y_ref;
        x_ref = &x;
        y_ref = &y;
    };

    0x0::Transaction::assert(*x_ref == 2, 42);
    y_ref;
}
}
