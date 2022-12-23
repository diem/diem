module 0x42::VerifyLoopsWithMemoryOps {

    use Std::Vector;
    spec module {
        pragma verify=true;
    }

    public fun nested_loop1(a: &mut vector<u64>, b: &mut vector<u64>) {
        let length = Vector::length(a);
        spec {
            assume length > 0;
            assume length == len(b);
        };
        let i = 0;
        while ({
            spec {
                invariant length == len(a);
                invariant length == len(b);
                invariant i <= length;
                invariant forall n in 0..i: a[n] == b[n];
            };
            (i < length)
        }) {
            let x = Vector::borrow_mut(a, i);
            let y = Vector::borrow_mut(b, i);
            loop {
                loop {
                    if (*x <= *y) {
                        break
                    };
                    *y = *y + 1;
                };
                if (*y <= *x) {
                    break
                };
                *x = *x + 1;
            };
        };
        spec {
            assert forall m in 0..length: a[m] == b[m];
        };
    }
    spec nested_loop1 {
        aborts_if false;
    }

    // This is equivalent to nested_loop1, but is much harder to verify, as we
    // don't have a way to specify the following loop invariants after havocing
    // - x points to a[i];
    // - y points to b[i];
    // The points-to relation of x and y is totally distorted after havoc.
    //
    // TODO (mengxu) find a way to specify the points-to relation, possibly
    // via introducing a new bytecode / call-operation named "PointerOf", which
    // takes a mutable reference and returns its "location" and "path".
    public fun nested_loop2(a: &mut vector<u64>, b: &mut vector<u64>) {
        let length = Vector::length(a);
        spec {
            assume length > 0;
            assume length == len(b);
        };
        let i = 0;
        let x = Vector::borrow_mut(a, i);
        let y = Vector::borrow_mut(b, i);
        loop {
            spec {
                invariant length == len(a);
                invariant length == len(b);
                invariant i < length;
                invariant forall n in 0..i: a[n] == b[n];
            };
            loop {
                loop {
                    if (*x <= *y) {
                        break
                    };
                    *y = *y + 1;
                };

                if (*y <= *x) {
                    break
                };
                *x = *x + 1;
            };
            i = i + 1;
            if (i == length) {
                break
            };
            x = Vector::borrow_mut(a, i);
            y = Vector::borrow_mut(b, i);
        };
        spec {
            assert forall m in 0..length: a[m] == b[m];
        };
    }
    spec nested_loop2 {
        aborts_if false;
    }
}
