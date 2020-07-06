module TestVector {
    use 0x1::Vector;
    struct T {
        x: u64,
    }

    // PR #4856 fixes the issue
    public fun update_ith_elem(v: &mut vector<T>, i: u64): bool {
        let size = Vector::length(v);
        if (i >= size) {
            return false
        };
        let elem = Vector::borrow_mut(v, i);
        let int_ref = &mut elem.x;

        *int_ref = 42;
        spec {
            assert int_ref == v[i].x;
        };
        true
    }
}
