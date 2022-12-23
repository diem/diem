// dep: ../../move-stdlib/modules/Vector.move

module 0x1::Borrow {
    // ensure that borrows get counted as reads when appropriate
    use Std::Vector;

    struct S has key { }

    // expected: read a/S
    fun borrow_s(a: address) acquires S {
        _ = borrow_global<S>(a)
    }

    // expected: read a/S
    fun borrow_s_mut(a: address) acquires S {
        _ = borrow_global_mut<S>(a)
    }

    // expected: read v/size
    fun borrow_vec(v: &vector<u64>) {
        let _ = Vector::borrow(v, 7);
    }

    // expected: read v/size
    fun borrow_vec_mut(v: &mut vector<u64>) {
        let _ = Vector::borrow_mut<u64>(v, 7);
    }
}
