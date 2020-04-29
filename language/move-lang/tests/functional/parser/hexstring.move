script {
use 0x0::Transaction;
use 0x0::Vector;

fun main() {
    let v = x"01020304";
    let sum: u64 = 0;
    while (!Vector::is_empty(&v)) {
        sum = sum + (Vector::pop_back(&mut v) as u64);
    };
    Transaction::assert(sum == 10, sum);
}
}
