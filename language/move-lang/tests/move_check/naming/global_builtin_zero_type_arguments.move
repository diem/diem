module M {
    resource struct R1 {}
    foo() {
        borrow_global<>(0x1);
        exists<>(0x1);
        R1 {} = move_from<>(0x1);
        move_to_sender<>(R1{});
    }
}
