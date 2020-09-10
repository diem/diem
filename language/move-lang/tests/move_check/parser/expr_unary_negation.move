script {

fun main() {
    // Unary negation is not supported.
    assert(((1 - -2) == 3) && (-(1 - 2) == 1), 100);
}
}
