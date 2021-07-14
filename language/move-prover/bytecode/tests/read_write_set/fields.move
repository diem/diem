address 0x2 {
module Fields {
    struct S { f: u64 }

    struct T<phantom A> { f: u64 }

    public fun borrow_read(s: &S): u64 {
        s.f
    }

    public fun borrow_read_generic<A>(t: &T<A>): u64 {
        t.f
    }
}
}
