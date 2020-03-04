use 0x0::Transaction;

fun main() {
    // Unary negation is not supported.
    Transaction::assert(((1 - -2) == 3) && (-(1 - 2) == 1), 100);
}
