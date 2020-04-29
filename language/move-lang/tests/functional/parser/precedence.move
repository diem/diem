script {
use 0x0::Transaction;

fun main() {
    Transaction::assert(true || true && false, 99); // "&&" has precedence over "||"
    Transaction::assert(true != false && false != true, 100); // "&&" has precedence over comparisons
    Transaction::assert(1 | 3 ^ 1 == 3, 101); // binary XOR has precedence over OR
    Transaction::assert(2 ^ 3 & 1 == 3, 102); // binary AND has precedence over XOR
    Transaction::assert(3 & 3 + 1 == 0, 103); // addition has precedence over binary AND
    Transaction::assert(1 + 2 * 3 == 7, 104); // multiplication has precedence over addition
}
}
