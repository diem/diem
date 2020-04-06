module M {
    use 0x0::Vector;
    use 0x0::Transaction;

    struct Foo {}
    resource struct Bar {}

    fun test_natives<T>(x1: T, x2: T): (T, T) {
        let v: vector<T> = Vector::empty();
        Transaction::assert(Vector::length(&v) == 0, 100);
        Vector::push_back(&mut v, x1);
        Transaction::assert(Vector::length(&v) == 1, 101);
        Vector::push_back(&mut v, x2);
        Transaction::assert(Vector::length(&v) == 2, 102);
        Vector::swap(&mut v, 0, 1);
        x1 = Vector::pop_back(&mut v);
        Transaction::assert(Vector::length(&v) == 1, 103);
        x2 = Vector::pop_back(&mut v);
        Transaction::assert(Vector::length(&v) == 0, 104);
        Vector::destroy_empty(v);
        (x1, x2)
    }

    public fun test() {
        test_natives<u8>(1u8, 2u8);
        test_natives<u64>(1u64, 2u64);
        test_natives<u128>(1u128, 2u128);
        test_natives<bool>(true, false);
        test_natives<address>(0x1, 0x2);

        test_natives<vector<u8>>(Vector::empty(), Vector::empty());

        test_natives<Foo>(Foo {}, Foo {});
        (Bar {}, Bar {}) = test_natives<Bar>(Bar {}, Bar {});
    }
}
// check: EXECUTED


//! new-transaction
use {{default}}::M;

fun main() {
    M::test();
}
// check: EXECUTED
