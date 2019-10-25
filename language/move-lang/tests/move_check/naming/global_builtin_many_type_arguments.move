module M {
    resource struct R1 {}
    resource struct R2 {}
    resource struct R3 {}
    resource struct R4 {}
    resource struct R5 {}
    foo() acquires R1 {
        borrow_global<R1, R2>(0x1);
        exists<R1, R2, R3>(0x1);
        R1 {} = move_from<R1, R2, R3, R4>(0x1);
        move_to_sender<R1, R2, R3, R4, R5>(R1{});

    }
}
