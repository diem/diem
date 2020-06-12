script {
use 0x0::Vector;
fun main() {
    let vec = Vector::empty();
    let (has, index) = Vector::index_of(&vec, &true);
    assert(!has, 0);
    assert(index == 0, 1);

    Vector::push_back<bool>(&mut vec, false);

    let (has, index) = Vector::index_of(&vec, &true);
    assert(!has, 2);
    assert(index == 0, 3);

    Vector::push_back<bool>(&mut vec, true);
    let (has, index) = Vector::index_of(&vec, &true);
    assert(has, 4);
    assert(index == 1, 5);

    Vector::push_back<bool>(&mut vec, true);
    let (has, index) = Vector::index_of(&vec, &true);
    assert(has, 6);
    assert(index == 1, 7);
}
}
