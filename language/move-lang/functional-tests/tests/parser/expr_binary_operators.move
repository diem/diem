script {
use 0x0::Transaction;

fun main() {
    Transaction::assert(1 == 1, 101);
    Transaction::assert(2 != 3, 102);
    Transaction::assert((3 < 4) && !(3 < 3), 103);
    Transaction::assert((4 > 3) && !(4 > 4), 104);
    Transaction::assert((5 <= 6) && (5 <= 5), 105);
    Transaction::assert((6 >= 5) && (6 >= 6), 106);
    Transaction::assert((true || false) && (false || true), 107);
    Transaction::assert((2 ^ 3) == 1, 108);
    Transaction::assert((1 | 2) == 3, 109);
    Transaction::assert((2 & 3) == 2, 110);
    Transaction::assert((2 << 1) == 4, 111);
    Transaction::assert((8 >> 2) == 2, 112);
    Transaction::assert((1 + 2) == 3, 113);
    Transaction::assert((3 - 2) == 1, 114);
    Transaction::assert((2 * 3) == 6, 115);
    Transaction::assert((9 / 3) == 3, 116);
    Transaction::assert((8 % 3) == 2, 117);
}
}
