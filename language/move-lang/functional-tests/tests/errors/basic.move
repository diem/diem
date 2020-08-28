script {
use 0x1::Errors;
fun main() {
    assert(Errors::invalid_state(0) == 1, 0);
    assert(Errors::requires_address(0) == 2, 1);
    assert(Errors::requires_role(0) == 3, 2);
    assert(Errors::not_published(0) == 5, 4);
    assert(Errors::already_published(0) == 6, 5);
    assert(Errors::invalid_argument(0) == 7, 6);
    assert(Errors::limit_exceeded(0) == 8, 7);
    assert(Errors::internal(0) == 10, 8);
    assert(Errors::custom(0) == 255, 9);
}
}
// check: "Keep(EXECUTED)"
