address 0x2 {
module Fields {
    resource struct S { f: u64 }

    resource struct T<A> { f: u64 }

    public fun borrow_read(s: &S): u64 {
        s.f
    }

    public fun borrow_read_generic<A>(t: &T<A>): u64 {
        t.f
    }
}
}
