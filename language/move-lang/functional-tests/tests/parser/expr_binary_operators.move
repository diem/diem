script {

fun main() {
    assert(1 == 1, 101);
    assert(2 != 3, 102);
    assert((3 < 4) && !(3 < 3), 103);
    assert((4 > 3) && !(4 > 4), 104);
    assert((5 <= 6) && (5 <= 5), 105);
    assert((6 >= 5) && (6 >= 6), 106);
    assert((true || false) && (false || true), 107);
    assert((2 ^ 3) == 1, 108);
    assert((1 | 2) == 3, 109);
    assert((2 & 3) == 2, 110);
    assert((2 << 1) == 4, 111);
    assert((8 >> 2) == 2, 112);
    assert((1 + 2) == 3, 113);
    assert((3 - 2) == 1, 114);
    assert((2 * 3) == 6, 115);
    assert((9 / 3) == 3, 116);
    assert((8 % 3) == 2, 117);
}
}
