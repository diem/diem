module {{default}}::Nop{
    public fun nop() { }
}

//! new-transaction
//! max-gas: 700
script {
use {{default}}::Nop;
use Std::Vector;
fun main() {
    Nop::nop();
    let v = Vector::empty();
    let vec_len = 1000;

    let i = 0;
    while (i < vec_len) {
        Vector::push_back(&mut v, i);
        i = i + 1;
    };

    i = 0;

    while (i < vec_len / 2) {
        Vector::swap(&mut v, i, vec_len - i - 1);
        i = i + 1;
    };
}
}
// check: "EXECUTION_FAILURE { status_code: OUT_OF_GAS, location: Script,"
// check: "gas_used: 700,"
// check: "Keep(OUT_OF_GAS)"
