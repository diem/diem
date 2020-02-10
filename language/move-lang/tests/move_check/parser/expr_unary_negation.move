use 0x0::Transaction;

fun main() {
    // Unary negation is not yet supported but still test that it parses.
    Transaction::assert(((1 - -2) == 3) && (-(1 - 2) == 1), 100);
}
