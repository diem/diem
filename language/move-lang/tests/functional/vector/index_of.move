script {
use 0x0::Vector;
use 0x0::Transaction;
fun main() {
    let vec = Vector::empty();
    let (has, index) = Vector::index_of(&vec, &true);
    Transaction::assert(!has, 0);
    Transaction::assert(index == 0, 1);

    Vector::push_back<bool>(&mut vec, false);

    let (has, index) = Vector::index_of(&vec, &true);
    Transaction::assert(!has, 2);
    Transaction::assert(index == 0, 3);

    Vector::push_back<bool>(&mut vec, true);
    let (has, index) = Vector::index_of(&vec, &true);
    Transaction::assert(has, 4);
    Transaction::assert(index == 1, 5);

    Vector::push_back<bool>(&mut vec, true);
    let (has, index) = Vector::index_of(&vec, &true);
    Transaction::assert(has, 6);
    Transaction::assert(index == 1, 7);
}
}
