module M {
    resource struct R1 {}
    resource struct R2 {}

    foo1(a: address) acquires R1 {
        borrow_global<R1>(a);
    }

    foo2(a: address) acquires R2 {
        borrow_global<R2>(a);
    }

    t0(a: address) acquires R1, R2 {
        borrow_global<R1>(a);
        borrow_global<R2>(a);
    }

    t1(a: address) acquires R1, R2 {
        borrow_global_mut<R1>(a);
        borrow_global_mut<R2>(a);
    }

    t2(a: address) acquires R1, R2 {
        R1{} = move_from<R1>(a);
        R2{} = borrow_global_mut<R2>(a);
    }

    t3(a: address) acquires R1, R2 {
        foo1(a);
        foo2(a);
    }

    t4(a: address) {
        exists<R1>(a);
        exists<R2>(a);
        move_to_sender<R1>(R1{});
        move_to_sender<R2>(R2{});
    }

}
