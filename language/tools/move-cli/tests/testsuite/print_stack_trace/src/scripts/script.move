script {
use Std::Debug;
use Std::Vector;
use 0x2::N;

fun main() {
    let v = Vector::empty();
    Vector::push_back(&mut v, true);
    Vector::push_back(&mut v, false);
    let r = Vector::borrow(&mut v, 1);
    let x = N::foo<bool, u64>();
    Debug::print(&x);
    _ = r;
    N::foo<u8,bool>();
}
}
